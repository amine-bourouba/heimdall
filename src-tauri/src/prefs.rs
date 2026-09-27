//! Preferences storage through the store plugin (ADR 0007): one typed [`Prefs`] persisted as
//! `prefs.json` in the OS app-config folder, plus the folder pickers `add_memory_folder`,
//! `remove_memory_folder` and `pick_config_project` open from Rust so the webview never supplies
//! a path (ADR 0010, rule 5). Never touches `~/.claude` — the Claude home and the app's own
//! preferences are different roots entirely.

use heimdall_core::models::prefs::{PreferredEditor, PreferredTerminal, Prefs, PrefsPatch};
use heimdall_core::paths::TargetOs;
use heimdall_core::{AppError, Result, WriteOutcome};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "prefs.json";

/// `Prefs.schemaVersion` v1 (today's only version) needs no migration; a future bump adds its own
/// arm here before the stored object is merged against current defaults (ADR 0007 decision 1).
fn migrate(stored: serde_json::Value) -> serde_json::Value {
    stored
}

/// In test builds only, an app that `.manage()`s this gets its own isolated store file instead of
/// the real, shared `STORE_FILE` under `BaseDirectory::AppData` — every `prefs.rs` test otherwise
/// resolves to the exact same real on-disk path (`mock_context` always uses the same identifier),
/// so cargo's default parallel test runner corrupts one test's expectations with another's writes.
/// Confirmed by a real cross-OS CI run: this crate's own tests had never actually executed before
/// (an earlier, now-fixed heimdall-core bug always aborted `cargo test --workspace` first).
#[cfg(test)]
struct TestStorePath(std::path::PathBuf);

fn store<R: Runtime>(app: &AppHandle<R>) -> Result<std::sync::Arc<tauri_plugin_store::Store<R>>> {
    #[cfg(test)]
    if let Some(path) = app.try_state::<TestStorePath>() {
        return app.store(path.0.clone()).map_err(|e| AppError::io(e.to_string()));
    }
    app.store(STORE_FILE).map_err(|e| AppError::io(e.to_string()))
}

/// Reads every key currently in the store as one flat JSON object — the same shape
/// `Prefs::from_stored` expects, and the same shape the store persists to disk as.
pub fn load<R: Runtime>(app: &AppHandle<R>) -> Result<Prefs> {
    let store = store(app)?;
    let object: serde_json::Map<String, serde_json::Value> = store.entries().into_iter().collect();
    Ok(Prefs::from_stored(
        migrate(serde_json::Value::Object(object)),
        TargetOs::current(),
    ))
}

/// Replaces every key in the store with `prefs`'s own fields, so no stale key from an older
/// schema lingers, then saves to disk.
pub fn save<R: Runtime>(app: &AppHandle<R>, prefs: &Prefs) -> Result<()> {
    let store = store(app)?;
    store.clear();
    let serde_json::Value::Object(map) = serde_json::to_value(prefs).expect("Prefs always serialises") else {
        unreachable!("Prefs serialises to an object")
    };
    for (key, value) in map {
        store.set(key, value);
    }
    store.save().map_err(|e| AppError::io(e.to_string()))
}

/// `update_prefs`'s validation: `fontScale` in range and on a 0.05 step, and every enum whose
/// valid set depends on the OS. The other enums (`colorTheme`, `sessionLayout`,
/// `globalSearchMode`) have no per-OS restriction, so the wire type itself — deserialisation
/// already rejects any value outside their fixed variant set — is the only validation they need.
fn validate_patch(patch: &PrefsPatch, os: TargetOs) -> Result<()> {
    if let Some(scale) = patch.font_scale {
        validate_font_scale(scale)?;
    }
    if let Some(editor) = patch.text_editor
        && !PreferredEditor::options(os).contains(&editor)
    {
        return Err(AppError::invalid_input(format!("{editor:?} is not offered on this OS")));
    }
    if let Some(terminal) = patch.preferred_terminal
        && !PreferredTerminal::options(os).contains(&terminal)
    {
        return Err(AppError::invalid_input(format!(
            "{terminal:?} is not offered on this OS"
        )));
    }
    Ok(())
}

const FONT_SCALE_MIN: f64 = 0.75;
const FONT_SCALE_MAX: f64 = 1.5;
const FONT_SCALE_STEP: f64 = 0.05;
// Repeatedly adding FONT_SCALE_STEP (real cross-OS CI: macos-latest/ubuntu-22.04) drifts the
// last valid step a few ULPs past FONT_SCALE_MAX before this ever reaches the step check below,
// which already tolerates exactly this much drift -- the range check needs the same tolerance.
const FONT_SCALE_EPSILON: f64 = 1e-9;

fn validate_font_scale(scale: f64) -> Result<()> {
    if scale < FONT_SCALE_MIN - FONT_SCALE_EPSILON || scale > FONT_SCALE_MAX + FONT_SCALE_EPSILON {
        return Err(AppError::invalid_input(format!(
            "fontScale must be between {FONT_SCALE_MIN} and {FONT_SCALE_MAX}"
        )));
    }
    let steps = (scale - FONT_SCALE_MIN) / FONT_SCALE_STEP;
    if (steps - steps.round()).abs() > FONT_SCALE_EPSILON {
        return Err(AppError::invalid_input("fontScale must be a multiple of 0.05"));
    }
    Ok(())
}

/// Applies every field the patch sets onto `prefs`; fields left `None` keep their current value.
fn apply_patch(mut prefs: Prefs, patch: PrefsPatch) -> Prefs {
    let PrefsPatch {
        show_menu_bar_icon,
        has_completed_onboarding,
        color_theme,
        font_scale,
        session_layout,
        config_layouts,
        show_agent_sessions,
        text_editor,
        preferred_terminal,
        open_terminal_on_bash,
        claude_code_path,
        show_animations,
        wrap_code_lines,
        auto_expand_blocks,
        parse_markdown_in_results,
        global_search_mode,
    } = patch;
    if let Some(v) = show_menu_bar_icon {
        prefs.show_menu_bar_icon = v;
    }
    if let Some(v) = has_completed_onboarding {
        prefs.has_completed_onboarding = v;
    }
    if let Some(v) = color_theme {
        prefs.color_theme = v;
    }
    if let Some(v) = font_scale {
        prefs.font_scale = v;
    }
    if let Some(v) = session_layout {
        prefs.session_layout = v;
    }
    if let Some(v) = config_layouts {
        prefs.config_layouts = v;
    }
    if let Some(v) = show_agent_sessions {
        prefs.show_agent_sessions = v;
    }
    if let Some(v) = text_editor {
        prefs.text_editor = v;
    }
    if let Some(v) = preferred_terminal {
        prefs.preferred_terminal = v;
    }
    if let Some(v) = open_terminal_on_bash {
        prefs.open_terminal_on_bash = v;
    }
    if let Some(v) = claude_code_path {
        prefs.claude_code_path = Some(v);
    }
    if let Some(v) = show_animations {
        prefs.show_animations = v;
    }
    if let Some(v) = wrap_code_lines {
        prefs.wrap_code_lines = v;
    }
    if let Some(v) = auto_expand_blocks {
        prefs.auto_expand_blocks = v;
    }
    if let Some(v) = parse_markdown_in_results {
        prefs.parse_markdown_in_results = v;
    }
    if let Some(v) = global_search_mode {
        prefs.global_search_mode = v;
    }
    prefs
}

/// `update_prefs`: validate, merge onto the stored preferences, save, and return the result.
pub fn update<R: Runtime>(app: &AppHandle<R>, patch: PrefsPatch) -> Result<Prefs> {
    validate_patch(&patch, TargetOs::current())?;
    let updated = apply_patch(load(app)?, patch);
    save(app, &updated)?;
    Ok(updated)
}

/// Index into `Prefs.customMemoryFolders` the contract's `folder:<index>` `MemorySourceId` form
/// names (§4); the sibling `project:<ProjectId>` form is never removable through this command.
fn folder_index(source_id: &str) -> Result<usize> {
    source_id
        .strip_prefix("folder:")
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| AppError::invalid_input(format!("not a custom-folder source id: {source_id}")))
}

/// `add_memory_folder`: opens a folder picker; a folder already present (compared the same way
/// Poirot's `standardizedFileURL` de-dup did) is a no-op, matching `MemorySourcesStore.add`.
pub fn add_memory_folder<R: Runtime>(app: &AppHandle<R>, picked: Option<String>) -> Result<WriteOutcome> {
    let Some(folder) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    let os = TargetOs::current();
    let mut prefs = load(app)?;
    if prefs
        .custom_memory_folders
        .iter()
        .any(|f| heimdall_core::paths::same_path(f, &folder, os))
    {
        return Ok(WriteOutcome::Unchanged);
    }
    prefs.custom_memory_folders.push(folder);
    save(app, &prefs)?;
    reregister_watched_folders(app, &prefs);
    Ok(WriteOutcome::Written)
}

/// `remove_memory_folder`: `sourceId` must be a `folder:<index>` id already in the list.
pub fn remove_memory_folder<R: Runtime>(app: &AppHandle<R>, source_id: &str) -> Result<WriteOutcome> {
    let index = folder_index(source_id)?;
    let mut prefs = load(app)?;
    if index >= prefs.custom_memory_folders.len() {
        return Err(AppError::not_found(format!("no memory source at {source_id}")));
    }
    prefs.custom_memory_folders.remove(index);
    save(app, &prefs)?;
    reregister_watched_folders(app, &prefs);
    Ok(WriteOutcome::Written)
}

/// Tells the watcher (M1-47) about the new folder list, if one is running — absent in tests that
/// build an `AppState` without also calling `watcher::init`, and that is fine: this is the
/// live-update path, not a correctness dependency (the folders are read fresh from `Prefs` by
/// every command that needs them regardless).
fn reregister_watched_folders<R: Runtime>(app: &AppHandle<R>, prefs: &Prefs) {
    if let Some(watcher) = app.try_state::<crate::watcher::WatcherHandle>() {
        watcher.set_custom_folders(&prefs.custom_memory_folders);
    }
}

/// `pick_config_project`: sets `configProjectPath` to the picked folder, or leaves it unchanged
/// if the picker is dismissed.
pub fn pick_config_project<R: Runtime>(app: &AppHandle<R>, picked: Option<String>) -> Result<WriteOutcome> {
    let Some(folder) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    let mut prefs = load(app)?;
    prefs.config_project_path = Some(folder);
    save(app, &prefs)?;
    Ok(WriteOutcome::Written)
}

/// Opens the native "choose a folder" dialog and blocks the calling (blocking-pool) thread until
/// the user answers — the command handlers call this via `state::blocking` before calling
/// `add_memory_folder`/`pick_config_project`, keeping the picked path out of the async event loop.
pub fn pick_folder<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::models::prefs::ColorThemeId;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        let app = mock_builder()
            .plugin(tauri_plugin_store::Builder::default().build())
            .build(mock_context(noop_assets()))
            .expect("build mock app");
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        app.manage(TestStorePath(
            std::env::temp_dir().join(format!("heimdall-test-prefs-{id}.json")),
        ));
        app
    }

    #[test]
    fn load_with_no_store_file_returns_defaults() {
        let app = app();
        assert_eq!(load(app.handle()).unwrap(), Prefs::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let app = app();
        let prefs = Prefs {
            color_theme: ColorThemeId::Solarized,
            custom_memory_folders: vec!["/tmp/a".into()],
            ..Default::default()
        };
        save(app.handle(), &prefs).unwrap();
        assert_eq!(load(app.handle()).unwrap(), prefs);
    }

    #[test]
    fn save_clears_a_key_from_an_older_schema() {
        let app = app();
        let store = store(app.handle()).unwrap();
        store.set("someRetiredKey", "x");
        store.save().unwrap();
        save(app.handle(), &Prefs::default()).unwrap();
        assert!(!store.has("someRetiredKey"));
    }

    #[test]
    fn migrate_v1_is_the_identity() {
        let v = serde_json::json!({"schemaVersion": 1, "fontScale": 1.25});
        assert_eq!(migrate(v.clone()), v);
    }

    #[test]
    fn update_merges_only_the_patched_fields() {
        let app = app();
        let patch = PrefsPatch {
            font_scale: Some(1.25),
            ..Default::default()
        };
        let updated = update(app.handle(), patch).unwrap();
        assert_eq!(
            (updated.font_scale, updated.color_theme),
            (1.25, ColorThemeId::default())
        );
        assert_eq!(load(app.handle()).unwrap().font_scale, 1.25);
    }

    #[test]
    fn update_rejects_a_font_scale_outside_the_range() {
        let app = app();
        for bad in [0.5, 1.6, -1.0] {
            let patch = PrefsPatch {
                font_scale: Some(bad),
                ..Default::default()
            };
            assert!(matches!(
                update(app.handle(), patch).unwrap_err(),
                AppError::InvalidInput { .. }
            ));
        }
    }

    #[test]
    fn update_rejects_a_font_scale_off_the_005_step() {
        let app = app();
        let patch = PrefsPatch {
            font_scale: Some(1.03),
            ..Default::default()
        };
        assert!(matches!(
            update(app.handle(), patch).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn update_accepts_every_valid_step() {
        let app = app();
        let mut scale = FONT_SCALE_MIN;
        while scale <= FONT_SCALE_MAX + 1e-9 {
            let patch = PrefsPatch {
                font_scale: Some(scale),
                ..Default::default()
            };
            update(app.handle(), patch).unwrap();
            scale += FONT_SCALE_STEP;
        }
    }

    #[test]
    fn update_rejects_xcode_off_macos() {
        if TargetOs::current() == TargetOs::Macos {
            return; // the restriction only bites on the OSes that do not offer Xcode
        }
        let app = app();
        let patch = PrefsPatch {
            text_editor: Some(PreferredEditor::Xcode),
            ..Default::default()
        };
        assert!(matches!(
            update(app.handle(), patch).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn update_leaves_the_store_untouched_when_validation_fails() {
        let app = app();
        update(
            app.handle(),
            PrefsPatch {
                font_scale: Some(1.0),
                ..Default::default()
            },
        )
        .unwrap();
        let before = load(app.handle()).unwrap();
        let bad = PrefsPatch {
            font_scale: Some(99.0),
            ..Default::default()
        };
        assert!(update(app.handle(), bad).is_err());
        assert_eq!(load(app.handle()).unwrap(), before);
    }

    // Ported from PoirotTests/App/MemorySourcesStoreTests.swift.
    #[test]
    fn add_memory_folder_appends_and_persists() {
        let app = app();
        add_memory_folder(app.handle(), Some("/tmp/poirot-a".into())).unwrap();
        assert_eq!(load(app.handle()).unwrap().custom_memory_folders, ["/tmp/poirot-a"]);
    }

    #[test]
    fn add_memory_folder_ignores_duplicates() {
        let app = app();
        add_memory_folder(app.handle(), Some("/tmp/poirot-a".into())).unwrap();
        let outcome = add_memory_folder(app.handle(), Some("/tmp/poirot-a".into())).unwrap();
        assert_eq!(outcome, WriteOutcome::Unchanged);
        assert_eq!(load(app.handle()).unwrap().custom_memory_folders.len(), 1);
    }

    #[test]
    fn add_memory_folder_cancelled_picker_changes_nothing() {
        let app = app();
        assert_eq!(add_memory_folder(app.handle(), None).unwrap(), WriteOutcome::Cancelled);
        assert!(load(app.handle()).unwrap().custom_memory_folders.is_empty());
    }

    #[test]
    fn remove_memory_folder_drops_the_folder_at_its_index() {
        let app = app();
        add_memory_folder(app.handle(), Some("/tmp/poirot-a".into())).unwrap();
        add_memory_folder(app.handle(), Some("/tmp/poirot-b".into())).unwrap();
        remove_memory_folder(app.handle(), "folder:0").unwrap();
        assert_eq!(load(app.handle()).unwrap().custom_memory_folders, ["/tmp/poirot-b"]);
    }

    #[test]
    fn remove_memory_folder_rejects_an_unresolvable_id() {
        let app = app();
        assert!(remove_memory_folder(app.handle(), "project:abc").is_err());
        assert!(remove_memory_folder(app.handle(), "folder:0").is_err()); // list is empty
    }

    #[test]
    fn pick_config_project_sets_the_path() {
        let app = app();
        pick_config_project(app.handle(), Some("/tmp/proj".into())).unwrap();
        assert_eq!(
            load(app.handle()).unwrap().config_project_path,
            Some("/tmp/proj".to_string())
        );
    }

    #[test]
    fn pick_config_project_cancelled_picker_changes_nothing() {
        let app = app();
        assert_eq!(
            pick_config_project(app.handle(), None).unwrap(),
            WriteOutcome::Cancelled
        );
        assert_eq!(load(app.handle()).unwrap().config_project_path, None);
    }

    /// The store plugin resolves `prefs.json` under the OS app-config directory (`BaseDirectory::AppData`);
    /// this asserts the resolved path's own text never contains a `.claude` segment, which is the one
    /// thing this task could get wrong that would put preferences under the Claude home.
    #[test]
    fn the_store_path_never_lands_under_the_claude_home() {
        let app = app();
        let path = tauri_plugin_store::resolve_store_path(app.handle(), STORE_FILE).unwrap();
        let text = path.to_string_lossy().to_lowercase();
        assert!(
            !text.contains(".claude"),
            "prefs store path must not be under .claude: {text}"
        );
    }
}
