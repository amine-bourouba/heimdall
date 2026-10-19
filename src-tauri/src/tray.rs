//! Tray icon and tray window (port-plan P-15, ADR 0003's own Tray row).
//!
//! Windows and macOS: a small borderless window (label `tray`, this task's own AC1) the tray
//! icon's left click shows/hides, positioned from the icon's own rect. Linux is not a smaller
//! version of the same thing -- `tauri::tray::TrayIconEvent`'s own doc comment says so directly:
//! "Linux: Unsupported. The event is not emitted even though the icon is shown and will still
//! show a context menu on right click." There is no click-to-toggle-a-popover event to handle at
//! all on that platform, so a native menu (Open Heimdall, up to 10 recent sessions, Quit) is not a
//! simplified fallback, it is the only mechanism Linux's own tray actually offers (AC2).

use heimdall_core::data::tray as tray_data;
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::discover;
use heimdall_core::search::{NavScreen, NavTarget};
use tauri::menu::{IsMenuItem, Menu, MenuItem, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::webview::WebviewWindowBuilder;
use tauri::{Manager, PhysicalPosition, Rect, WebviewUrl, WindowEvent};

use crate::commands::tray::open_in_main_window_inner;
use crate::state::AppState;

const WIDTH: f64 = 320.0;
const HEIGHT: f64 = 440.0;
const MAX_RECENT_MENU_ITEMS: usize = 10;

/// Holds the tray icon so `prefs::update` can show/hide it live when `showMenuBarIcon` changes
/// (AC3) -- the same managed-state pattern `watcher::WatcherHandle` already uses for its own
/// "preference command reaches back into a long-lived handle" need.
pub struct TrayHandle {
    icon: TrayIcon,
}

impl TrayHandle {
    pub fn set_visible(&self, visible: bool) {
        let _ = self.icon.set_visible(visible);
    }
}

pub fn init(app: &mut tauri::App) -> tauri::Result<()> {
    if cfg!(target_os = "linux") {
        init_linux(app)
    } else {
        init_window_tray(app)
    }
}

fn show_menu_bar_icon(app: &tauri::App) -> bool {
    crate::prefs::load(app.handle())
        .map(|p| p.show_menu_bar_icon)
        .unwrap_or(true)
}

/// `position`/`size` arrive in whatever unit the OS reported (`Rect`'s own fields are the
/// `dpi::Position`/`dpi::Size` enums, physical *or* logical depending on platform) -- converted to
/// physical pixels via the window's own scale factor before any arithmetic, the same conversion
/// `tauri::Rect`'s own doc examples use. macOS's menu bar sits at the screen's top, so the popover
/// opens *below* the icon; every other platform's tray lives at the bottom (Windows taskbar), so
/// it opens *above*. Centred horizontally on the icon.
fn position_near_tray(window: &tauri::WebviewWindow, rect: &Rect) -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let icon_pos = rect.position.to_physical::<f64>(scale);
    let icon_size = rect.size.to_physical::<f64>(scale);
    let window_size = window.outer_size()?;
    let x = icon_pos.x + icon_size.width / 2.0 - f64::from(window_size.width) / 2.0;
    let y = if cfg!(target_os = "macos") {
        icon_pos.y + icon_size.height
    } else {
        icon_pos.y - f64::from(window_size.height)
    };
    window.set_position(PhysicalPosition::new(x as i32, y as i32))
}

fn init_window_tray(app: &mut tauri::App) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, "tray", WebviewUrl::App("tray.html".into()))
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(true)
        .always_on_top(true)
        .visible(false)
        .build()?;

    // Hidden on blur (AC1): losing focus (clicking anywhere else) hides the popover, matching
    // every native menu-bar-extra/tray-popover convention this is standing in for.
    let tray_window = app.get_webview_window("tray").expect("just built above");
    let hide_on_blur = tray_window.clone();
    tray_window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            let _ = hide_on_blur.hide();
        }
    });

    let mut builder = TrayIconBuilder::new().on_tray_icon_event(|tray, event| {
        let TrayIconEvent::Click {
            rect,
            button: MouseButton::Left,
            button_state: MouseButtonState::Down,
            ..
        } = event
        else {
            return;
        };
        let app = tray.app_handle();
        let Some(window) = app.get_webview_window("tray") else {
            return;
        };
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = position_near_tray(&window, &rect);
            let _ = window.show();
            let _ = window.set_focus();
        }
    });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let tray = builder.build(app)?;
    tray.set_visible(show_menu_bar_icon(app))?;
    app.manage(TrayHandle { icon: tray });
    Ok(())
}

/// Builds `label` as a tray menu item and records what it should navigate to (or `None` for
/// "Quit", which has no target). Kept as one small helper rather than hand-writing the same
/// `MenuItemBuilder::with_id(...).build(app)?` three times with slightly different ids.
fn menu_item(app: &tauri::App, id: &str, label: &str) -> tauri::Result<MenuItem<tauri::Wry>> {
    MenuItemBuilder::with_id(id, label).build(app)
}

fn init_linux(app: &mut tauri::App) -> tauri::Result<()> {
    let claude_home = &app.state::<AppState>().claude_home;
    let projects = discover(&claude_home.projects_dir(), TargetOs::current())
        .map(|loaded| loaded.value)
        .unwrap_or_default();
    let recent = tray_data::recent_sessions(&projects);

    let mut items = vec![menu_item(app, "tray-open", "Open Heimdall")?];
    // Index -> target, so the menu-event handler below can map an id like "tray-session-3" back
    // to the session it names without re-encoding ids/UUIDs into the menu item id itself.
    let mut targets: Vec<NavTarget> = Vec::new();
    for (project, session) in recent.iter().take(MAX_RECENT_MENU_ITEMS) {
        let id = format!("tray-session-{}", targets.len());
        items.push(menu_item(app, &id, &session.title())?);
        targets.push(NavTarget::Session {
            project_id: project.id.clone(),
            session_id: session.id.clone(),
        });
    }
    items.push(menu_item(app, "tray-quit", "Quit")?);

    let refs: Vec<&dyn IsMenuItem<tauri::Wry>> = items.iter().map(|i| i as &dyn IsMenuItem<_>).collect();
    let menu = Menu::with_items(app, &refs)?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().0.as_str() {
            "tray-open" => {
                let _ = open_in_main_window_inner(
                    app,
                    NavTarget::Screen {
                        screen: NavScreen::Sessions,
                    },
                );
            }
            "tray-quit" => app.exit(0),
            id => {
                if let Some(target) = id
                    .strip_prefix("tray-session-")
                    .and_then(|n| n.parse::<usize>().ok())
                    .and_then(|i| targets.get(i))
                {
                    let _ = open_in_main_window_inner(app, target.clone());
                }
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let tray = builder.build(app)?;
    tray.set_visible(show_menu_bar_icon(app))?;
    app.manage(TrayHandle { icon: tray });
    Ok(())
}
