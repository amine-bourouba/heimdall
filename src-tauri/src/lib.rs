//! Heimdall desktop shell: plugins, command wiring and startup (ADR 0002).
//! Each module below is owned by one task; see docs/tasks/README.md.

mod commands;
mod events;
mod menu;
mod platform;
mod prefs;
mod state;
mod tray;
mod updater;
mod watcher;
mod window;

pub fn run() {
    let specta = commands::builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            window::init(app)?;
            menu::init(app)?;
            watcher::init(app)?;
            tray::init(app)?;
            updater::init(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Heimdall");
}
