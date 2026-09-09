//! Model tool handler for reading the current session todo list.

use crate::agentic::coordination::get_global_coordinator;
use crate::agentic::core::{CompressedTodoSnapshot, CompressionEntry, Message, MessageHelper};
use crate::agentic::tools::framework::{Tool, ToolResult, ToolUseContext};
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use serde_json::{json, Value};

const GET_TODO_TOOL_NAME: &str = "get_todo";

/// get_todo tool - read the current session todo list.
pub struct GetTodoTool;

impl GetTodoTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GetTodoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GetTodoTool {
    fn name(&self) -> &str {
        GET_TODO_TOOL_NAME
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok(
            "Read the current todo list for this session. Returns the most recent todo list \
created or updated with TodoWrite, including each item's id, content, and status (pending, \
in_progress, or completed)."
                .to_string(),
        )
    }

    fn short_description(&self) -> String {
        "Read the current session todo list.".to_string()
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
        context: &ToolUseContext,
    ) -> BitFunResult<Vec<ToolResult>> {
        let session_id = context
            .session_id
            .clone()
            .ok_or_else(|| BitFunError::validation("session_id is unavailable"))?;
        let coordinator = get_global_coordinator()
            .ok_or_else(|| BitFunError::validation("coordinator is unavailable"))?;
        let messages = coordinator
            .get_session_manager()
            .get_context_messages(&session_id)
            .await?;

        let (data, result_for_assistant) = build_todo_result(latest_todo_snapshot(&messages));
        Ok(vec![ToolResult::Result {
            data,
            result_for_assistant: Some(result_for_assistant),
            image_attachments: None,
        }])
    }
}

fn latest_todo_snapshot(messages: &[Message]) -> Option<CompressedTodoSnapshot> {
    MessageHelper::get_last_todo_snapshot(messages).or_else(|| {
        // A compressed turn may carry the last todo snapshot after the raw
        // TodoWrite tool call has been folded into the compression payload.
        messages.iter().rev().find_map(|message| {
            let payload = message.metadata.compression_payload.as_ref()?;
            payload.entries.iter().rev().find_map(|entry| match entry {
                CompressionEntry::Turn { todo: Some(todo), .. } => Some(todo.clone()),
                _ => None,
            })
        })
    })
}

fn build_todo_result(snapshot: Option<CompressedTodoSnapshot>) -> (Value, String) {
    match snapshot {
        Some(snapshot) => {
            let todos: Vec<Value> = snapshot
                .todos
                .into_iter()
                .map(|item| {
                    let mut object = serde_json::Map::new();
                    if let Some(id) = item.id {
                        object.insert("id".to_string(), json!(id));
                    }
                    object.insert("content".to_string(), json!(item.content));
                    object.insert("status".to_string(), json!(item.status));
                    Value::Object(object)
                })
                .collect();
            let count = todos.len();
            let summary = snapshot
                .summary
                .unwrap_or_else(|| format!("Current todo list has {count} task(s)"));
            (json!({ "todos": todos, "count": count, "summary": summary }), summary)
        }
        None => {
            let summary = "No todo list has been created in this session.".to_string();
            (
                json!({ "todos": [], "count": 0, "summary": summary }),
                summary,
            )
        }
    }
}
