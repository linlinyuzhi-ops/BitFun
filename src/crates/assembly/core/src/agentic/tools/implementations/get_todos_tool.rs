//! Model tool handler for reading the scheduled todos shown in the Todos panel.

use crate::agentic::tools::framework::{Tool, ToolResult, ToolUseContext};
use crate::service::cron::{
    get_global_cron_service, CronJob, CronJobCompletionStatus, CronJobHandling, CronJobTarget,
    CronSchedule,
};
use crate::util::errors::{OpenBitFunError, OpenBitFunResult};
use async_trait::async_trait;
use serde_json::{json, Value};

const GET_TODOS_TOOL_NAME: &str = "get_todos";

/// get_todos tool - read the scheduled todos across all workspaces.
///
/// This is the data shown in the Todos panel (scheduled jobs), not the session
/// TodoWrite checklist. Read the shared cron service so the assistant and the
/// panel always observe the same set of tasks.
pub struct GetTodosTool;

impl GetTodosTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GetTodosTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GetTodosTool {
    fn name(&self) -> &str {
        GET_TODOS_TOOL_NAME
    }

    async fn description(&self) -> OpenBitFunResult<String> {
        Ok(
            "Read the scheduled todos shown in the Todos panel. These are scheduled jobs \
(cron jobs) across all workspaces, not the session TodoWrite checklist. Each todo includes its \
name, schedule (at/every/cron), enabled flag, completion status (pending/in_progress/completed), \
handling (agent/manual), target (session or workspace), payload text, planned start/completion \
times, and the next run time. Use this to report or act on the user's scheduled tasks."
                .to_string(),
        )
    }

    fn short_description(&self) -> String {
        "Read the scheduled todos across all workspaces.".to_string()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {}
        })
    }

    fn is_readonly(&self) -> bool {
        true
    }

    async fn call_impl(
        &self,
        _input: &Value,
        _context: &ToolUseContext,
    ) -> OpenBitFunResult<Vec<ToolResult>> {
        let service = get_global_cron_service()
            .ok_or_else(|| OpenBitFunError::tool("cron service not initialized".to_string()))?;
        let jobs: Vec<CronJob> = service.list_jobs().await;

        let serialized_jobs: Vec<Value> = jobs
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .map_err(|error| OpenBitFunError::serialization(error.to_string()))?;

        let result_for_assistant = build_result_for_assistant(&jobs);
        Ok(vec![ToolResult::Result {
            data: json!({
                "success": true,
                "count": jobs.len(),
                "jobs": serialized_jobs,
            }),
            result_for_assistant: Some(result_for_assistant),
            image_attachments: None,
        }])
    }
}

fn build_result_for_assistant(jobs: &[CronJob]) -> String {
    if jobs.is_empty() {
        return "No scheduled todos found.".to_string();
    }

    let mut lines = vec![format!("Found {} scheduled todo(s):", jobs.len())];
    for job in jobs {
        lines.push(format!(
            "- {} [{}] ({}, {}) | {} | {}",
            job.name,
            if job.enabled { "enabled" } else { "disabled" },
            completion_status_label(job.completion_status),
            handling_label(job.handling),
            schedule_label(&job.schedule),
            target_label(job),
        ));
    }
    lines.join("\n")
}

fn completion_status_label(status: CronJobCompletionStatus) -> &'static str {
    match status {
        CronJobCompletionStatus::Pending => "pending",
        CronJobCompletionStatus::InProgress => "in_progress",
        CronJobCompletionStatus::Completed => "completed",
    }
}

fn handling_label(handling: CronJobHandling) -> &'static str {
    match handling {
        CronJobHandling::Agent => "agent",
        CronJobHandling::Manual => "manual",
    }
}

fn schedule_label(schedule: &CronSchedule) -> String {
    match schedule {
        CronSchedule::At { at } => format!("at {}", at),
        CronSchedule::Every { every_ms, .. } => format!("every {}s", every_ms.div_ceil(1_000)),
        CronSchedule::Cron { expr, tz } => match tz.as_deref() {
            Some(tz) if !tz.trim().is_empty() => format!("cron {} ({})", expr, tz),
            _ => format!("cron {}", expr),
        },
    }
}

fn target_label(job: &CronJob) -> String {
    match &job.target {
        CronJobTarget::Session { session_id, .. } => format!("session {}", session_id),
        CronJobTarget::Workspace { workspace, .. } => {
            format!("workspace {}", workspace.workspace_path)
        }
    }
}
