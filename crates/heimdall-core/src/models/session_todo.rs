//! Todo items from `~/.claude/todos/<sessionId>[-agent-<agentId>].json` (Models/SessionTodo.swift).

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::Loaded;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub content: String,
    pub active_form: String,
    pub status: TodoStatus,
}

/// Read as `in_progress` from disk, sent as `inProgress` over IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum TodoStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename(serialize = "inProgress", deserialize = "in_progress"))]
    InProgress,
    #[serde(rename = "pending")]
    Pending,
}

impl TodoItem {
    /// Stable id from the content (Poirot used `content.hashValue`, which changes between runs):
    /// 64-bit FNV-1a, so the same content always gets the same id.
    pub fn id(&self) -> u64 {
        crate::stable_hash(self.content.as_bytes())
    }

    /// Decode a todo file's array item by item: an item that does not parse (for example an
    /// unknown status) is skipped and counted instead of failing the whole file.
    pub fn list_from_json(text: &str) -> serde_json::Result<Loaded<Vec<TodoItem>>> {
        let raw: Vec<serde_json::Value> = serde_json::from_str(text)?;
        let total = raw.len();
        let items: Vec<TodoItem> = raw.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
        let skipped = (total - items.len()) as u32;
        Ok(Loaded::new(items, skipped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(json: &str) -> TodoItem {
        serde_json::from_str(json).unwrap()
    }

    // Ported from PoirotTests/Models/SessionTodoTests.swift.
    #[test]
    fn decodes_completed_status() {
        let t = one(r#"{"content": "Fix the bug", "status": "completed", "activeForm": "Fixing the bug"}"#);
        assert_eq!((t.content.as_str(), t.status, t.active_form.as_str()), ("Fix the bug", TodoStatus::Completed, "Fixing the bug"));
    }
    #[test]
    fn decodes_in_progress_status() {
        assert_eq!(one(r#"{"content": "Write tests", "status": "in_progress", "activeForm": "Writing tests"}"#).status, TodoStatus::InProgress);
    }
    #[test]
    fn decodes_pending_status() {
        assert_eq!(one(r#"{"content": "Deploy", "status": "pending", "activeForm": "Deploying"}"#).status, TodoStatus::Pending);
    }
    #[test]
    fn decodes_array_of_todos() {
        let list = TodoItem::list_from_json(r#"[
            {"content": "Task A", "status": "completed", "activeForm": "Completing A"},
            {"content": "Task B", "status": "in_progress", "activeForm": "Working on B"},
            {"content": "Task C", "status": "pending", "activeForm": "Starting C"}]"#).unwrap();
        let statuses: Vec<_> = list.value.iter().map(|t| t.status).collect();
        assert_eq!(statuses, [TodoStatus::Completed, TodoStatus::InProgress, TodoStatus::Pending]);
    }
    #[test]
    fn identifiable_id_derived_from_content() {
        let t = one(r#"{"content": "Do something", "status": "pending", "activeForm": "Doing something"}"#);
        assert_eq!(t.id(), one(r#"{"content": "Do something", "status": "completed", "activeForm": "x"}"#).id());
    }
    #[test]
    fn hashable_todos_with_same_content_are_equal() {
        let a = one(r#"{"content": "Same task", "status": "completed", "activeForm": "Completing"}"#);
        let b = one(r#"{"content": "Same task", "status": "pending", "activeForm": "Starting"}"#);
        assert_eq!(a.id(), b.id());
    }
    #[test]
    fn decodes_empty_array() {
        assert!(TodoItem::list_from_json("[]").unwrap().value.is_empty());
    }

    #[test]
    fn unknown_status_is_skipped_not_fatal_and_ipc_uses_camel_case() {
        let list = TodoItem::list_from_json(r#"[{"content": "a", "status": "blocked", "activeForm": "x"},
            {"content": "b", "status": "in_progress", "activeForm": "y"}]"#).unwrap();
        assert_eq!((list.value.len(), list.skipped), (1, 1));
        assert_eq!(serde_json::to_value(&list.value[0]).unwrap()["status"], "inProgress");
    }
}
