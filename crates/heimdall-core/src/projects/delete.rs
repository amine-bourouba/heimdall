//! Session and project deletes inside the Claude home (AppState.swift:380-465's own
//! `deleteSession`/`deleteProject`): a session's own transcript file plus its matching todo files
//! -- Swift's own `deleteSession` never removes todos at all, a gap this task's own Goal
//! ("sessions (file plus todos)") asks to close, using `TodoLoader.deleteTodos`'s own prefix
//! match (`data::todos`'s own matching helpers stay private to that module; only its `AGENT_INFIX`
//! constant is shared here, `pub(crate)`, to keep the one literal in one place) -- and a whole
//! project folder, which never touches todos either (matching Swift's own `deleteProject` as-is).
//!
//! Sub-agent sessions nested under `<project>/<id>/subagents/` (`projects::subagents`) are out of
//! scope: there is no UI entry point to delete one independently of its parent, matching Swift's
//! own `deleteSession`, which only ever removes a top-level session's own file.
//!
//! This task's own AC2 ("deletes refuse any path outside the Claude home"): both `project_id` and
//! `session_id` are validated as plain file names first (no `..`, no separators), and the path
//! built from them is canonicalised and checked against `projects_dir` before any removal
//! (`paths::ensure_within_root`, shared with `config::write::files`'s own M3-42 symlink-escape
//! guard) -- a symlink planted at the exact expected location can't redirect a delete elsewhere.
//! A todo file is never checked this way: it is only ever reached by listing `todos_dir` itself,
//! and `remove_file` on a symlink removes the symlink, never the file it points to.

use std::io::ErrorKind;
use std::path::Path;

use crate::data::todos::AGENT_INFIX;
use crate::error::{AppError, Result, WriteOutcome};
use crate::paths::{self, ensure_within_root};

/// Removes `<projects_dir>/<project_id>/<session_id>.jsonl` and every todo file under `todos_dir`
/// matching `session_id` (the plain file plus any `<session_id>-agent-*` file). Either side being
/// already gone is fine; `Unchanged` only when neither existed.
pub fn delete_session(
    projects_dir: &Path,
    todos_dir: &Path,
    project_id: &str,
    session_id: &str,
) -> Result<WriteOutcome> {
    let project_id = paths::plain_file_name(project_id)?;
    let session_id = paths::plain_file_name(session_id)?;

    let mut outcome = WriteOutcome::Unchanged;
    let session_file = projects_dir.join(project_id).join(format!("{session_id}.jsonl"));
    if session_file.exists() {
        let safe = ensure_within_root(&session_file, projects_dir)?;
        std::fs::remove_file(&safe).map_err(|e| AppError::from_io(&e, &safe))?;
        outcome = WriteOutcome::Written;
    }
    if delete_todos(todos_dir, session_id)? {
        outcome = WriteOutcome::Written;
    }
    Ok(outcome)
}

/// Removes the whole `<projects_dir>/<project_id>` folder.
pub fn delete_project(projects_dir: &Path, project_id: &str) -> Result<WriteOutcome> {
    let project_id = paths::plain_file_name(project_id)?;
    let dir = projects_dir.join(project_id);
    if !dir.is_dir() {
        return Ok(WriteOutcome::Unchanged);
    }
    let safe = ensure_within_root(&dir, projects_dir)?;
    std::fs::remove_dir_all(&safe).map_err(|e| AppError::from_io(&e, &safe))?;
    Ok(WriteOutcome::Written)
}

/// `true` if any file was removed. A missing `todos_dir` is not an error
/// (`TodoLoaderTests.deleteTodos_nonExistentPath_doesNotCrash`).
fn delete_todos(todos_dir: &Path, session_id: &str) -> Result<bool> {
    let agent_prefix = format!("{session_id}{AGENT_INFIX}");
    let entries = match std::fs::read_dir(todos_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(AppError::from_io(&e, todos_dir)),
    };
    let mut deleted_any = false;
    for path in entries.flatten().map(|e| e.path()) {
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem == session_id || stem.starts_with(&agent_prefix) {
            std::fs::remove_file(&path).map_err(|e| AppError::from_io(&e, &path))?;
            deleted_any = true;
        }
    }
    Ok(deleted_any)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirs() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let projects = tmp.path().join("projects");
        let todos = tmp.path().join("todos");
        std::fs::create_dir_all(&projects).unwrap();
        std::fs::create_dir_all(&todos).unwrap();
        (tmp, projects, todos)
    }

    // Ported from PoirotTests/Services/TodoLoaderTests.swift's `deleteTodos` cases.
    #[test]
    fn delete_session_removes_the_file_and_its_matching_todos() {
        let (_tmp, projects, todos) = dirs();
        let dir = projects.join("proj");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("s1.jsonl"), "{}").unwrap();
        std::fs::write(todos.join("s1.json"), "[]").unwrap();
        std::fs::write(todos.join("s1-agent-a1.json"), "[]").unwrap();
        std::fs::write(todos.join("other-session.json"), "[]").unwrap();

        assert_eq!(
            delete_session(&projects, &todos, "proj", "s1").unwrap(),
            WriteOutcome::Written
        );
        assert!(!dir.join("s1.jsonl").exists());
        assert!(!todos.join("s1.json").exists());
        assert!(!todos.join("s1-agent-a1.json").exists());
        assert!(todos.join("other-session.json").exists());
    }

    #[test]
    fn delete_session_leaves_unrelated_files_alone() {
        let (_tmp, projects, todos) = dirs();
        let dir = projects.join("proj");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("s1.jsonl"), "{}").unwrap();
        std::fs::write(dir.join("s2.jsonl"), "{}").unwrap();
        std::fs::write(todos.join("s2.json"), "[]").unwrap();

        delete_session(&projects, &todos, "proj", "s1").unwrap();
        assert!(dir.join("s2.jsonl").exists() && todos.join("s2.json").exists());
    }

    #[test]
    fn delete_session_nonexistent_session_is_unchanged() {
        let (_tmp, projects, todos) = dirs();
        std::fs::create_dir(projects.join("proj")).unwrap();
        assert_eq!(
            delete_session(&projects, &todos, "proj", "ghost").unwrap(),
            WriteOutcome::Unchanged
        );
    }

    #[test]
    fn delete_session_missing_todos_dir_does_not_crash() {
        let (tmp, projects, _todos) = dirs();
        let missing_todos = tmp.path().join("no-such-todos-dir");
        let dir = projects.join("proj");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("s1.jsonl"), "{}").unwrap();
        assert_eq!(
            delete_session(&projects, &missing_todos, "proj", "s1").unwrap(),
            WriteOutcome::Written
        );
    }

    #[test]
    fn delete_session_rejects_a_traversal_attempt() {
        let (_tmp, projects, todos) = dirs();
        assert!(matches!(
            delete_session(&projects, &todos, "..", "s1").unwrap_err(),
            AppError::InvalidInput { .. }
        ));
        assert!(matches!(
            delete_session(&projects, &todos, "proj", "../../escape").unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn delete_project_removes_the_whole_folder() {
        let (_tmp, projects, _todos) = dirs();
        let dir = projects.join("proj");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("s1.jsonl"), "{}").unwrap();
        std::fs::write(dir.join("sessions-index.json"), "{}").unwrap();

        assert_eq!(delete_project(&projects, "proj").unwrap(), WriteOutcome::Written);
        assert!(!dir.exists());
    }

    #[test]
    fn delete_project_nonexistent_is_unchanged() {
        let (_tmp, projects, _todos) = dirs();
        assert_eq!(delete_project(&projects, "ghost").unwrap(), WriteOutcome::Unchanged);
    }

    #[test]
    fn delete_project_rejects_a_traversal_attempt() {
        let (_tmp, projects, _todos) = dirs();
        assert!(matches!(
            delete_project(&projects, "../outside").unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn delete_session_refuses_a_symlink_escaping_the_project_dir() {
        let (tmp, projects, todos) = dirs();
        let outside = tmp.path().join("outside.jsonl");
        std::fs::write(&outside, "do not delete me").unwrap();
        let dir = projects.join("proj");
        std::fs::create_dir(&dir).unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("s1.jsonl")).unwrap();

        assert!(delete_session(&projects, &todos, "proj", "s1").is_err());
        assert!(outside.exists());
    }

    #[cfg(unix)]
    #[test]
    fn delete_project_refuses_a_symlinked_project_folder() {
        let (tmp, projects, _todos) = dirs();
        let outside = tmp.path().join("outside-dir");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("keep"), "do not delete me").unwrap();
        std::os::unix::fs::symlink(&outside, projects.join("proj")).unwrap();

        assert!(delete_project(&projects, "proj").is_err());
        assert!(outside.join("keep").exists());
    }
}
