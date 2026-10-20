//! Native macOS app menu emitting `menu-action` events (the Swift reference app's PoirotApp.swift:47-104,
//! docs/ipc-contract.md §7).
//!
//! Windows and Linux keep no native menu bar (this task's own goal) -- every action here is
//! already reachable through `useShortcuts` (M2-08) on every OS, so nothing is lost by skipping
//! the menu bar on those platforms. The frontend listener that turns a received `menu-action`
//! event into `useAppActions().dispatch(...)` does not exist yet -- that is the real app root's
//! job (M2-09 builds it), not this task's: `useAppActions` itself is a plain dispatch registry
//! with no IPC subscription of its own (`useAppActions.ts`), and M2-13's own AC only asks what
//! this module emits, not who consumes it.
//!
//! `"?"` has no accelerator string here (the Help item) because muda's own accelerator parser
//! has no `Code` variant for it -- confirmed by reading `muda-0.19.3`'s `accelerator.rs`
//! `parse_code` match arms directly rather than guessing a syntax that would panic at startup.

#[cfg(target_os = "macos")]
use heimdall_core::app_shell::AppAction;
#[cfg(target_os = "macos")]
use tauri::menu::{Menu, MenuItemBuilder, SubmenuBuilder};
#[cfg(target_os = "macos")]
use tauri_specta::Event;

#[cfg(target_os = "macos")]
use crate::events::MenuAction;

#[cfg(target_os = "macos")]
pub fn init(app: &mut tauri::App) -> tauri::Result<()> {
    let app_menu = SubmenuBuilder::new(app, "Heimdall")
        .item(&MenuItemBuilder::with_id("about", "About Heimdall").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("checkUpdates", "Check for Updates…").build(app)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("Cmd+,")
                .build(app)?,
        )
        .separator()
        .quit()
        .build()?;

    // Cut/copy/paste/select all (AC1) -- needed for text fields, since the webview has no native
    // Edit menu of its own on macOS without one being supplied here.
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(
            &MenuItemBuilder::with_id("fontIncrease", "Increase Font Size")
                .accelerator("Cmd+=")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("fontDecrease", "Decrease Font Size")
                .accelerator("Cmd+-")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("fontReset", "Reset Font Size")
                .accelerator("Cmd+0")
                .build(app)?,
        )
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&MenuItemBuilder::with_id("help", "Heimdall Help").build(app)?)
        .item(&MenuItemBuilder::with_id("shortcuts", "Keyboard Shortcuts").build(app)?)
        .build()?;

    Menu::with_items(app, &[&app_menu, &edit_menu, &help_menu])?.set_as_app_menu()?;

    app.on_menu_event(|app_handle, event| {
        // AC2: every custom item's id above is the exact `AppAction` wire name (contract
        // ipc-contract.md §7); the predefined cut/copy/paste/select-all/quit items carry no id
        // of ours and fall through the wildcard untouched.
        let action = match event.id().0.as_str() {
            "about" => AppAction::About,
            "checkUpdates" => AppAction::CheckUpdates,
            "settings" => AppAction::Settings,
            "fontIncrease" => AppAction::FontIncrease,
            "fontDecrease" => AppAction::FontDecrease,
            "fontReset" => AppAction::FontReset,
            "help" => AppAction::Help,
            "shortcuts" => AppAction::Shortcuts,
            _ => return,
        };
        let _ = MenuAction { action }.emit(app_handle);
    });

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn init(_app: &mut tauri::App) -> tauri::Result<()> {
    Ok(())
}
