//! Model tool handler for creating, updating, and deleting the scheduled todos
//! shown in the Todos panel.

use crate::agentic::tools::framework::{Tool, ToolResult, ToolUseContext};
use crate::agentic::workspace::WorkspaceBinding;
use crate::service::cron::{
    get_global_cron_service, CreateCronJobRequest, CronJob, CronJobCompletionStatus,
    CronJobHandling, CronJobPayload, CronJobTarget, CronLaunchSpec, CronSchedule,
    CronWorkspaceRef, UpdateCronJobRequest,
};
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use chrono::DateTime;
use serde::Deserialize;
use serde_json::{json, Value};

const MANAGE_TODOS_TOOL_NAME: &str = "manage_todos";
const DEFAULT_TODO_NAME: &str = "Todo";
const DEFAULT_AGENT_TYPE: &str = "agentic";

/// manage_todos tool - write access to the scheduled todos shown in the Todos
/// panel.
///
/// This is the same cron data source that get_todos reads and the panel edits,
/// so create/update/delete here are immediately reflected in both.
pub struct ManageTodosTool;

impl ManageTodosTool {
    pub fn new() -> Self {
        Self
    }

    fn workspace_ref_from_context_binding(binding: &WorkspaceBinding) -> CronWorkspaceRef {
        CronWorkspaceRef {
            workspace_id: binding.workspace_id.clone(),
            workspace_path: binding.root_path_string(),
            project_workspace_path: Some(binding.project_root_path_string()),
            execution_target: binding.execution_target.clone(),
            remote_connection_id: binding.connection_id().map(ToOwned::to_owned),
            remote_ssh_host: if binding.is_remote() {
                Some(binding.session_identity.hostname.clone())
                    .filter(|value| !value.trim().is_empty())
            } else {
                None
            },
        }
    }

    fn resolve_workspace_ref(context: &ToolUseContext) -> BitFunResult<CronWorkspaceRef> {
        let binding = context.workspace.as_ref().ok_or_else(|| {
            BitFunError::tool(
                "the current workspace is unavailable; a workspace is required to manage todos"
                    .to_string(),
            )
        })?;
        Ok(Self::workspace_ref_from_context_binding(binding))
    }

    fn parse_iso_timestamp_ms(value: &str, field: &str) -> BitFunResult<i64> {
        let parsed = DateTime::parse_from_rfc3339(value).map_err(|error| {
            BitFunError::tool(format!(
                "{} must be a valid ISO-8601 timestamp: {}",
                field, error
            ))
        })?;
        Ok(parsed.timestamp_millis())
    }

    fn normalize_name(name: Option<String>) -> String {
        match name {
            Some(name) if !name.trim().is_empty() => name.trim().to_string(),
            _ => DEFAULT_TODO_NAME.to_string(),
        }
    }

    fn normalize_optional_name(name: Option<String>) -> BitFunResult<Option<String>> {
        match name {
            Some(name) if name.trim().is_empty() => Err(BitFunError::tool(
                "name cannot be empty when provided".to_string(),
            )),
            Some(name) => Ok(Some(name.trim().to_string())),
            None => Ok(None),
        }
    }

    fn resolve_target(
        target: TodoTargetInput,
        context: &ToolUseContext,
        workspace_ref: CronWorkspaceRef,
    ) -> BitFunResult<CronJobTarget> {
        match target {
            TodoTargetInput::Session { session_id } => {
                let session_id = match session_id {
                    Some(id) if !id.trim().is_empty() => id.trim().to_string(),
                    _ => context.session_id.clone().ok_or_else(|| {
                        BitFunError::tool(
                            "sessionId is required when the current session is unavailable"
                                .to_string(),
                        )
                    })?,
                };
                bitfun_core_types::validate_session_id(&session_id).map_err(BitFunError::tool)?;
                Ok(CronJobTarget::Session {
                    session_id,
                    workspace: workspace_ref,
                })
            }
            TodoTargetInput::Workspace { agent_type } => {
                let agent_type = match agent_type {
                    Some(value) if !value.trim().is_empty() => value.trim().to_string(),
                    _ => DEFAULT_AGENT_TYPE.to_string(),
                };
                Ok(CronJobTarget::Workspace {
                    workspace: workspace_ref,
                    launch: CronLaunchSpec {
                        agent_type,
                        model_id: None,
                    },
                })
            }
        }
    }

    fn serialize_job(job: &CronJob) -> BitFunResult<Value> {
        serde_json::to_value(job).map_err(|error| BitFunError::serialization(error.to_string()))
    }
}

impl Default for ManageTodosTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ManageTodosAction {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManageTodosInput {
    action: ManageTodosAction,
    job_id: Option<String>,
    name: Option<String>,
    text: Option<String>,
    schedule: Option<TodoScheduleInput>,
    target: Option<TodoTargetInput>,
    enabled: Option<bool>,
    completion_status: Option<CronJobCompletionStatus>,
    handling: Option<CronJobHandling>,
    planned_start_at_ms: Option<Option<i64>>,
    planned_completion_at_ms: Option<Option<i64>>,
    actual_completion_at_ms: Option<Option<i64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum TodoScheduleInput {
    At {
        at: String,
    },
    Every {
        #[serde(rename = "everySeconds")]
        every_seconds: u64,
        anchor: Option<String>,
    },
    Cron {
        expr: String,
        tz: Option<String>,
    },
}

impl TodoScheduleInput {
    fn to_service_schedule(&self) -> BitFunResult<CronSchedule> {
        match self {
            Self::At { at } => {
                let at = at.trim();
                if at.is_empty() {
                    return Err(BitFunError::tool("schedule.at cannot be empty".to_string()));
                }
                ManageTodosTool::parse_iso_timestamp_ms(at, "schedule.at")?;
                Ok(CronSchedule::At { at: at.to_string() })
            }
            Self::Every {
                every_seconds,
                anchor,
            } => {
                if *every_seconds == 0 {
                    return Err(BitFunError::tool(
                        "schedule.everySeconds must be greater than 0".to_string(),
                    ));
                }
                let every_ms = every_seconds.checked_mul(1_000).ok_or_else(|| {
                    BitFunError::tool("schedule.everySeconds is too large".to_string())
                })?;
                let anchor_ms = match anchor.as_deref() {
                    Some(anchor) if anchor.trim().is_empty() => {
                        return Err(BitFunError::tool(
                            "schedule.anchor cannot be empty when provided".to_string(),
                        ));
                    }
                    Some(anchor) => Some(ManageTodosTool::parse_iso_timestamp_ms(
                        anchor.trim(),
                        "schedule.anchor",
                    )?),
                    None => None,
                };
                Ok(CronSchedule::Every {
                    every_ms,
                    anchor_ms,
                })
            }
            Self::Cron { expr, tz } => {
                let expr = expr.trim();
                if expr.is_empty() {
                    return Err(BitFunError::tool("schedule.expr cannot be empty".to_string()));
                }
                Ok(CronSchedule::Cron {
                    expr: expr.to_string(),
                    tz: tz.as_ref()
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty()),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum TodoTargetInput {
    Session {
        #[serde(rename = "sessionId")]
        session_id: Option<String>,
    },
    Workspace {
        #[serde(rename = "agentType")]
        agent_type: Option<String>,
    },
}

#[async_trait]
impl Tool for ManageTodosTool {
    fn name(&self) -> &str {
        MANAGE_TODOS_TOOL_NAME
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok(
            "Create, update, or delete the scheduled todos shown in the Todos panel. These are \
the same scheduled jobs (cron jobs) that get_todos reads; changes appear in the panel immediately. \
Use get_todos first to obtain the jobId of an existing todo you want to update or delete.

Actions:
- create: add a todo. Requires text and schedule. target is required; it may target the current \
session or a workspace.
- update: modify an existing todo by jobId. Provide only the fields to change.
- delete: remove a todo by jobId.

Schedule is one of:
- { kind: 'at', at: '<ISO-8601>' }
- { kind: 'every', everySeconds: <n>, anchor: '<optional ISO-8601>' }
- { kind: 'cron', expr: '<cron-expression>', tz: '<optional timezone>' }

Target is one of:
- { kind: 'session', sessionId: '<optional, defaults to current session>' }
- { kind: 'workspace', agentType: '<optional, defaults to agentic>' }

Other fields: name (default 'Todo'), text (todo description), enabled (boolean), completionStatus \
('pending' | 'in_progress' | 'completed'), handling ('agent' | 'manual'), and the planned/actual \
timestamps in epoch milliseconds."
                .to_string(),
        )
    }

    fn short_description(&self) -> String {
        "Create, update, or delete scheduled todos.".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "update", "delete"],
                    "description": "Action to perform."
                },
                "jobId": {
                    "type": "string",
                    "description": "Todo id. Required for update and delete."
                },
                "name": {
                    "type": "string",
                    "description": "Todo name. Defaults to 'Todo' on create."
                },
                "text": {
                    "type": "string",
                    "description": "Todo description / payload text. Required on create."
                },
                "schedule": {
                    "type": "object",
                    "description": "Required on create. One of: {\"kind\":\"at\",\"at\":\"<ISO-8601>\"}, {\"kind\":\"every\",\"everySeconds\":<n>,\"anchor\":\"<optional ISO-8601>\"}, or {\"kind\":\"cron\",\"expr\":\"<cron-expression>\",\"tz\":\"<optional timezone>\"}."
                },
                "target": {
                    "type": "object",
                    "description": "Required on create. One of: {\"kind\":\"session\",\"sessionId\":\"<optional, defaults to current session>\"} or {\"kind\":\"workspace\",\"agentType\":\"<optional, defaults to agentic>\"}."
                },
                "enabled": {
                    "type": "boolean",
                    "description": "Whether the todo is enabled. Defaults to true on create."
                },
                "completionStatus": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed"]
                },
                "handling": {
                    "type": "string",
                    "enum": ["agent", "manual"]
                },
                "plannedStartAtMs": {
                    "type": ["integer", "null"],
                    "description": "Epoch milliseconds. Omit to leave unchanged; null to clear."
                },
                "plannedCompletionAtMs": {
                    "type": ["integer", "null"],
                    "description": "Epoch milliseconds. Omit to leave unchanged; null to clear."
                },
                "actualCompletionAtMs": {
                    "type": ["integer", "null"],
                    "description": "Epoch milliseconds. Omit to leave unchanged; null to clear."
                }
            },
            "required": ["action"],
            "additionalProperties": false
        })
    }

    fn is_readonly(&self) -> bool {
        false
    }

    async fn call_impl(
        &self,
        input: &Value,
        context: &ToolUseContext,
    ) -> BitFunResult<Vec<ToolResult>> {
        let parsed: ManageTodosInput = serde_json::from_value(input.clone())
            .map_err(|error| BitFunError::tool(format!("Invalid input: {}", error)))?;
        let service = get_global_cron_service()
            .ok_or_else(|| BitFunError::tool("cron service not initialized".to_string()))?;

        match parsed.action {
            ManageTodosAction::Create => {
                let text = parsed
                    .text
                    .ok_or_else(|| BitFunError::tool("text is required for create".to_string()))?;
                if text.trim().is_empty() {
                    return Err(BitFunError::tool("text must not be empty".to_string()));
                }
                let schedule = parsed.schedule.ok_or_else(|| {
                    BitFunError::tool("schedule is required for create".to_string())
                })?;
                let target = parsed
                    .target
                    .ok_or_else(|| BitFunError::tool("target is required for create".to_string()))?;

                let workspace_ref = Self::resolve_workspace_ref(context)?;
                let target = Self::resolve_target(target, context, workspace_ref)?;

                let created = service
                    .create_job(CreateCronJobRequest {
                        name: Self::normalize_name(parsed.name),
                        schedule: schedule.to_service_schedule()?,
                        payload: CronJobPayload { text },
                        enabled: parsed.enabled.unwrap_or(true),
                        target,
                        completion_status: parsed.completion_status.unwrap_or_default(),
                        handling: parsed.handling.unwrap_or_default(),
                        planned_start_at_ms: parsed.planned_start_at_ms.flatten(),
                        planned_completion_at_ms: parsed.planned_completion_at_ms.flatten(),
                        actual_completion_at_ms: parsed.actual_completion_at_ms.flatten(),
                    })
                    .await?;

                let job = Self::serialize_job(&created)?;
                let result_for_assistant =
                    format!("Created scheduled todo '{}' ({}).", created.name, created.id);
                Ok(vec![ToolResult::Result {
                    data: json!({
                        "success": true,
                        "action": "create",
                        "job": job,
                    }),
                    result_for_assistant: Some(result_for_assistant),
                    image_attachments: None,
                }])
            }
            ManageTodosAction::Update => {
                let job_id = parsed
                    .job_id
                    .as_deref()
                    .ok_or_else(|| BitFunError::tool("jobId is required for update".to_string()))?;
                if job_id.trim().is_empty() {
                    return Err(BitFunError::tool("jobId cannot be empty".to_string()));
                }
                if let Some(text) = parsed.text.as_deref() {
                    if text.trim().is_empty() {
                        return Err(BitFunError::tool(
                            "text must not be empty when provided".to_string(),
                        ));
                    }
                }
                if parsed.name.is_none()
                    && parsed.text.is_none()
                    && parsed.schedule.is_none()
                    && parsed.enabled.is_none()
                    && parsed.completion_status.is_none()
                    && parsed.handling.is_none()
                    && parsed.planned_start_at_ms.is_none()
                    && parsed.planned_completion_at_ms.is_none()
                    && parsed.actual_completion_at_ms.is_none()
                {
                    return Err(BitFunError::tool(
                        "update requires at least one field to change".to_string(),
                    ));
                }

                let updated = service
                    .update_job(
                        job_id,
                        UpdateCronJobRequest {
                            name: Self::normalize_optional_name(parsed.name)?,
                            schedule: parsed
                                .schedule
                                .as_ref()
                                .map(|value| value.to_service_schedule())
                                .transpose()?,
                            payload: parsed.text.map(|text| CronJobPayload { text }),
                            enabled: parsed.enabled,
                            target: None,
                            completion_status: parsed.completion_status,
                            handling: parsed.handling,
                            planned_start_at_ms: parsed.planned_start_at_ms,
                            planned_completion_at_ms: parsed.planned_completion_at_ms,
                            actual_completion_at_ms: parsed.actual_completion_at_ms,
                        },
                    )
                    .await?;

                let job = Self::serialize_job(&updated)?;
                let result_for_assistant =
                    format!("Updated scheduled todo '{}' ({}).", updated.name, updated.id);
                Ok(vec![ToolResult::Result {
                    data: json!({
                        "success": true,
                        "action": "update",
                        "jobId": job_id,
                        "job": job,
                    }),
                    result_for_assistant: Some(result_for_assistant),
                    image_attachments: None,
                }])
            }
            ManageTodosAction::Delete => {
                let job_id = parsed
                    .job_id
                    .as_deref()
                    .ok_or_else(|| BitFunError::tool("jobId is required for delete".to_string()))?;
                if job_id.trim().is_empty() {
                    return Err(BitFunError::tool("jobId cannot be empty".to_string()));
                }

                let deleted = service.delete_job(job_id).await?;
                let result_for_assistant = if deleted {
                    format!("Deleted scheduled todo '{}'.", job_id)
                } else {
                    format!("No scheduled todo found for '{}'.", job_id)
                };
                Ok(vec![ToolResult::Result {
                    data: json!({
                        "success": true,
                        "action": "delete",
                        "jobId": job_id,
                        "deleted": deleted,
                    }),
                    result_for_assistant: Some(result_for_assistant),
                    image_attachments: None,
                }])
            }
        }
    }
}
