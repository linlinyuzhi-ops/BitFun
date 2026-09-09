//! Scheduled job data types.

use openbitfun_agent_runtime::scheduled_job::DEFAULT_SCHEDULED_JOB_RETRY_DELAY_MS;
pub use openbitfun_agent_runtime::scheduled_job::{
    ScheduledJobRunStatus as CronJobRunStatus, ScheduledJobRuntimeState as CronJobState,
};
use openbitfun_core_types::SessionExecutionTarget;
use serde::{Deserialize, Deserializer, Serialize};

pub const CRON_JOBS_VERSION: u32 = 2;

pub const DEFAULT_RETRY_DELAY_MS: i64 = DEFAULT_SCHEDULED_JOB_RETRY_DELAY_MS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobsFile {
    pub version: u32,
    pub jobs: Vec<CronJob>,
}

impl Default for CronJobsFile {
    fn default() -> Self {
        Self {
            version: CRON_JOBS_VERSION,
            jobs: Vec::new(),
        }
    }
}

/// User-facing lifecycle of a Todo, independent of the scheduler run state.
///
/// This is a manual marker set from the Todos panel; it does not change when a
/// job runs. The scheduler keeps driving a job according to `enabled` and its
/// schedule regardless of this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CronJobCompletionStatus {
    /// 待启动 — not started yet.
    #[default]
    Pending,
    /// 完成中 — currently being worked on.
    InProgress,
    /// 已完成 — done.
    Completed,
}

/// Who handles a Todo when it fires.
///
/// Agent jobs are dispatched to a session as usual. Manual jobs are never
/// dispatched; the scheduler only marks them due so the user can act on them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CronJobHandling {
    #[default]
    Agent,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: CronSchedule,
    pub payload: CronJobPayload,
    pub enabled: bool,
    pub target: CronJobTarget,
    pub created_at_ms: i64,
    pub config_updated_at_ms: i64,
    pub updated_at_ms: i64,
    #[serde(default)]
    pub state: CronJobState,
    #[serde(default)]
    pub completion_status: CronJobCompletionStatus,
    #[serde(default)]
    pub handling: CronJobHandling,
    /// Latest scheduled time a manual job became due; cleared once the user
    /// marks it completed. Agent jobs never set this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_due_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_start_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_completion_at_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_completion_at_ms: Option<i64>,
}

impl CronJob {
    pub fn is_one_shot(&self) -> bool {
        matches!(self.schedule, CronSchedule::At { .. })
    }

    pub fn target_kind(&self) -> CronJobTargetKind {
        self.target.kind()
    }

    pub fn workspace(&self) -> &CronWorkspaceRef {
        self.target.workspace()
    }

    pub fn session_id(&self) -> Option<&str> {
        self.target.session_id()
    }

    pub fn launch(&self) -> Option<&CronLaunchSpec> {
        self.target.launch()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CronJobTargetKind {
    Session,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronWorkspaceRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub workspace_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_workspace_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_target: Option<SessionExecutionTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_ssh_host: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronLaunchSpec {
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
}

impl Default for CronLaunchSpec {
    fn default() -> Self {
        Self {
            agent_type: default_agent_type(),
            model_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CronJobTarget {
    Session {
        #[serde(rename = "sessionId")]
        session_id: String,
        workspace: CronWorkspaceRef,
    },
    Workspace {
        workspace: CronWorkspaceRef,
        #[serde(default)]
        launch: CronLaunchSpec,
    },
}

impl CronJobTarget {
    pub fn kind(&self) -> CronJobTargetKind {
        match self {
            Self::Session { .. } => CronJobTargetKind::Session,
            Self::Workspace { .. } => CronJobTargetKind::Workspace,
        }
    }

    pub fn workspace(&self) -> &CronWorkspaceRef {
        match self {
            Self::Session { workspace, .. } | Self::Workspace { workspace, .. } => workspace,
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Session { session_id, .. } => Some(session_id.as_str()),
            Self::Workspace { .. } => None,
        }
    }

    pub fn launch(&self) -> Option<&CronLaunchSpec> {
        match self {
            Self::Session { .. } => None,
            Self::Workspace { launch, .. } => Some(launch),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CronSchedule {
    At {
        at: String,
    },
    Every {
        #[serde(rename = "everyMs")]
        every_ms: u64,
        #[serde(rename = "anchorMs", skip_serializing_if = "Option::is_none")]
        anchor_ms: Option<i64>,
    },
    Cron {
        expr: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tz: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobPayload {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCronJobRequest {
    pub name: String,
    pub schedule: CronSchedule,
    pub payload: CronJobPayload,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub target: CronJobTarget,
    #[serde(default)]
    pub completion_status: CronJobCompletionStatus,
    #[serde(default)]
    pub handling: CronJobHandling,
    #[serde(default)]
    pub planned_start_at_ms: Option<i64>,
    #[serde(default)]
    pub planned_completion_at_ms: Option<i64>,
    #[serde(default)]
    pub actual_completion_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCronJobRequest {
    pub name: Option<String>,
    pub schedule: Option<CronSchedule>,
    pub payload: Option<CronJobPayload>,
    pub enabled: Option<bool>,
    pub target: Option<CronJobTarget>,
    #[serde(default)]
    pub completion_status: Option<CronJobCompletionStatus>,
    #[serde(default)]
    pub handling: Option<CronJobHandling>,
    #[serde(default, deserialize_with = "deserialize_optional_ms")]
    pub planned_start_at_ms: Option<Option<i64>>,
    #[serde(default, deserialize_with = "deserialize_optional_ms")]
    pub planned_completion_at_ms: Option<Option<i64>>,
    #[serde(default, deserialize_with = "deserialize_optional_ms")]
    pub actual_completion_at_ms: Option<Option<i64>>,
}

const fn default_enabled() -> bool {
    true
}

fn default_agent_type() -> String {
    "agentic".to_string()
}

/// Deserializes an optional timestamp for `UpdateCronJobRequest` so that a
/// missing field means "leave unchanged" (`None`) while an explicit JSON
/// `null` means "clear" (`Some(None)`).
fn deserialize_optional_ms<'de, D>(deserializer: D) -> Result<Option<Option<i64>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer).map(Some)
}
