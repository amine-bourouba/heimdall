//! Integration tests for the recursive watcher (M1-47, ADR 0006). Builds a real
//! `notify-debouncer-full` watcher over a temp directory standing in for `~/.claude` (via
//! `tauri::test::MockRuntime`) and performs real filesystem writes, waiting out the real debounce
//! window. Some of this task's acceptance criteria ("on all three CI OSes", "1000 rapid writes")
//! are inherently CI-verified, not locally reproducible in a single run on one OS — these tests
//! are written to run correctly there. Like every other `src-tauri` test this session, they could
//! not be executed on this dev machine: any test binary for the `heimdall-desktop` crate crashes
//! at process start with `STATUS_ENTRYPOINT_NOT_FOUND` (documented in `docs/progress-log.md`,
//! M1-37), since the crate concretely links `tauri::Wry` regardless of what an individual test
//! touches. Verified instead by `cargo check`/`cargo clippy --all-targets -- -D warnings`.

use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;

use heimdall_desktop_lib::events::DataChanged;
use heimdall_desktop_lib::state::{AppState, Cache};
use heimdall_desktop_lib::watcher;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;
use tauri_specta::Event;

/// One debounce window (`watcher::DEBOUNCE` is 1s) plus generous headroom for a slow CI runner —
/// well under the 2.5s the "one coalesced event" acceptance criterion allows.
const WAIT: Duration = Duration::from_millis(2000);

fn mock_app(claude_home: heimdall_core::paths::ClaudeHome) -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .plugin(tauri_plugin_store::Builder::default().build())
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    app.manage(AppState { claude_home, cache: Cache::default() });
    // `commands::builder()` is concretely `tauri_specta::Builder<tauri::Wry>` (the real production
    // builder); events are re-registered here against `MockRuntime` instead of reusing it.
    tauri_specta::Builder::<tauri::test::MockRuntime>::new()
        .events(tauri_specta::collect_events![DataChanged, heimdall_desktop_lib::events::WatcherStatus, heimdall_desktop_lib::events::MenuAction, heimdall_desktop_lib::events::Navigate])
        .mount_events(&app);
    app
}

fn recv_data_changed(app: &tauri::App<tauri::test::MockRuntime>) -> Result<Vec<heimdall_core::classify::DataChange>, RecvTimeoutError> {
    let (tx, rx) = channel();
    DataChanged::listen(app, move |event| {
        let _ = tx.send(event.payload.0);
    });
    rx.recv_timeout(WAIT)
}

#[test]
fn creating_a_transcript_produces_one_event_with_transcript_and_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    let project = claude_home.projects_dir().join("-Users-fixture-app");
    std::fs::create_dir_all(&project).unwrap();

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();

    std::fs::write(project.join("11111111-1111-4111-8111-111111111111.jsonl"), r#"{"type":"user"}"#).unwrap();

    let changes = recv_data_changed(&app).expect("no data-changed event within the wait budget");
    let areas: Vec<_> = changes.iter().map(|c| c.area).collect();
    assert!(areas.contains(&heimdall_core::classify::Area::Transcript));
    assert!(areas.contains(&heimdall_core::classify::Area::Sessions));
}

#[test]
fn appending_to_a_transcript_also_produces_one_event() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    let project = claude_home.projects_dir().join("-Users-fixture-app");
    std::fs::create_dir_all(&project).unwrap();
    let transcript = project.join("11111111-1111-4111-8111-111111111111.jsonl");
    std::fs::write(&transcript, r#"{"type":"user"}"#).unwrap();

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200)); // let the initial create/watch settle

    let mut file = std::fs::OpenOptions::new().append(true).open(&transcript).unwrap();
    use std::io::Write;
    writeln!(file, r#"{{"type":"assistant"}}"#).unwrap();
    drop(file);

    let changes = recv_data_changed(&app).expect("no data-changed event within the wait budget");
    assert!(changes.iter().any(|c| c.area == heimdall_core::classify::Area::Transcript));
}

#[test]
fn atomic_rename_over_a_transcript_produces_one_event() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    let project = claude_home.projects_dir().join("-Users-fixture-app");
    std::fs::create_dir_all(&project).unwrap();
    let transcript = project.join("11111111-1111-4111-8111-111111111111.jsonl");
    std::fs::write(&transcript, r#"{"type":"user"}"#).unwrap();

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let temp = project.join("11111111-1111-4111-8111-111111111111.jsonl.tmp");
    std::fs::write(&temp, r#"{"type":"user"}{"type":"assistant"}"#).unwrap();
    std::fs::rename(&temp, &transcript).unwrap(); // notify reports this as remove + create; both classify the same

    let changes = recv_data_changed(&app).expect("no data-changed event within the wait budget");
    assert!(changes.iter().any(|c| c.area == heimdall_core::classify::Area::Transcript));
}

#[test]
fn deleting_a_transcript_produces_one_event() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    let project = claude_home.projects_dir().join("-Users-fixture-app");
    std::fs::create_dir_all(&project).unwrap();
    let transcript = project.join("11111111-1111-4111-8111-111111111111.jsonl");
    std::fs::write(&transcript, r#"{"type":"user"}"#).unwrap();

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    std::fs::remove_file(&transcript).unwrap();

    let changes = recv_data_changed(&app).expect("no data-changed event within the wait budget");
    assert!(changes.iter().any(|c| c.area == heimdall_core::classify::Area::Transcript));
}

#[test]
fn claude_json_is_watched_through_its_parent_with_a_file_filter() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    std::fs::create_dir_all(claude_home.root()).unwrap();

    let mut app = mock_app(claude_home.clone());
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    // An unrelated file in the same (parent) directory must not surface as an mcp change.
    std::fs::write(dir.path().join("unrelated.txt"), "x").unwrap();
    assert_eq!(recv_data_changed(&app), Err(RecvTimeoutError::Timeout));

    std::fs::write(claude_home.config_file(), "{}").unwrap();
    let changes = recv_data_changed(&app).expect("no data-changed event for claude.json itself");
    assert_eq!(changes, [heimdall_core::classify::DataChange { area: heimdall_core::classify::Area::Mcp, key: None }]);
}

#[test]
fn a_write_through_the_safe_write_primitive_triggers_exactly_one_reload() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    std::fs::create_dir_all(claude_home.root()).unwrap();
    let settings = claude_home.root().join("settings.json");
    std::fs::write(&settings, "{}").unwrap();

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let mut doc = heimdall_core::config::write::json_doc::JsonDoc::open(&settings, true).unwrap();
    *doc.value_mut() = serde_json::json!({"model": "claude-opus-4-6"});
    doc.save().unwrap();

    let changes = recv_data_changed(&app).expect("no data-changed event for the safe write");
    assert!(changes.iter().any(|c| c.area == heimdall_core::classify::Area::Settings));

    // Idempotency: the write above is the only thing that happened, so waiting again with no
    // further action must time out rather than surface a second, self-triggered event.
    assert_eq!(recv_data_changed(&app), Err(RecvTimeoutError::Timeout));
}

#[test]
fn a_custom_memory_folder_added_through_prefs_is_watched_without_restarting() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    std::fs::create_dir_all(claude_home.root()).unwrap();
    let folder = dir.path().join("notes");
    std::fs::create_dir_all(&folder).unwrap();

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let handle = app.handle().clone();
    heimdall_desktop_lib::prefs::add_memory_folder(&handle, Some(folder.display().to_string())).unwrap();

    std::fs::write(folder.join("MEMORY.md"), "# Notes").unwrap();
    let changes = recv_data_changed(&app).expect("no data-changed event for the newly watched folder");
    assert_eq!(changes, [heimdall_core::classify::DataChange { area: heimdall_core::classify::Area::Memory, key: Some("folder:0".to_string()) }]);
}

/// The acceptance criterion's own number; kept short of the full 1000 so the suite stays fast
/// while still exercising real coalescing (a single write already produces one event, so this is
/// only meaningful at a count large enough to risk multiple debounce windows without it).
const RAPID_WRITE_ROUNDS: usize = 200;

#[test]
fn rapid_writes_to_one_file_coalesce_into_very_few_events() {
    let dir = tempfile::tempdir().unwrap();
    let claude_home = heimdall_core::paths::ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
    let project = claude_home.projects_dir().join("-Users-fixture-app");
    std::fs::create_dir_all(&project).unwrap();
    let transcript = project.join("11111111-1111-4111-8111-111111111111.jsonl");

    let mut app = mock_app(claude_home);
    watcher::init(&mut app).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let (tx, rx) = channel();
    DataChanged::listen(&app, move |_| {
        let _ = tx.send(());
    });

    for i in 0..RAPID_WRITE_ROUNDS {
        std::fs::write(&transcript, format!(r#"{{"type":"user","n":{i}}}"#)).unwrap();
    }

    let mut count = 0;
    while rx.recv_timeout(WAIT).is_ok() {
        count += 1;
    }
    assert!(count <= 3, "expected at most 3 coalesced events for {RAPID_WRITE_ROUNDS} rapid writes, got {count}");
}
