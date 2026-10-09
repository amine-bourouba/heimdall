//! Main window creation, per-OS chrome and navigation guard (port-plan P-4, ADR 0010).
//!
//! The window is built programmatically here, not declared in `tauri.conf.json`, because the
//! navigation/new-window guards and macOS's title-bar style are only available through
//! `WebviewWindowBuilder` (task M2-12's own AC asks to "verify the handler API in the pinned
//! Tauri version" -- confirmed by reading `tauri-2.11.6`'s real source, not assumed from docs
//! that may have drifted: `WebviewWindowBuilder::on_navigation`/`on_new_window`,
//! `webview/webview_window.rs:266,315`).
//!
//! `title_bar_style`/`hidden_title` are themselves `#[cfg(target_os = "macos")]` in the tauri
//! crate -- they do not exist as methods at all on a non-macOS build. ADR 0003's general
//! preference is to pass an OS as data so the same code runs on every host; here that is not
//! possible, since the Rust *compiler* removes the method for other targets regardless of any
//! runtime `TargetOs` value this crate could pass in. The `#[cfg(target_os = "macos")]` split
//! below is a real constraint of the tauri API, not a stylistic choice.

use std::sync::Mutex;

use tauri::utils::config::WebviewUrl;
use tauri::webview::{NewWindowResponse, WebviewWindowBuilder};
use tauri::Url;

const TITLE: &str = "Heimdall";
const WIDTH: f64 = 1280.0;
const HEIGHT: f64 = 820.0;
const MIN_WIDTH: f64 = 900.0;
const MIN_HEIGHT: f64 = 600.0;

pub fn init(app: &mut tauri::App) -> tauri::Result<()> {
    // `on_navigation` fires for the webview's very first load too (confirmed: the Tauri docs'
    // own example checks url.scheme() unconditionally, with no special-casing for an initial
    // load, which only makes sense if the callback already runs then). There is no public API
    // to ask "what is this app's own origin" ahead of time (`Webview::is_local_url` exists but
    // is private, `webview/mod.rs:1698`) -- so the first call's own origin becomes the allowed
    // one, and every later call is checked against it. Stored as the ASCII serialization
    // (`url::Origin::ascii_serialization`, an inherent method, so no extra Cargo.toml dependency
    // is needed just to name the `Origin` type), not `url::Origin` itself -- the `url` crate is
    // only a transitive dependency here (pulled in through `tauri`), so this crate cannot name
    // its types directly.
    let allowed_origin: Mutex<Option<String>> = Mutex::new(None);

    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title(TITLE)
        .inner_size(WIDTH, HEIGHT)
        .min_inner_size(MIN_WIDTH, MIN_HEIGHT)
        .resizable(true)
        .on_navigation(move |url: &Url| {
            let this_origin = url.origin().ascii_serialization();
            let mut guard = allowed_origin.lock().expect("window navigation-origin lock");
            match guard.as_ref() {
                Some(allowed) => *allowed == this_origin,
                None => {
                    *guard = Some(this_origin);
                    true
                }
            }
        })
        // Refuses every `window.open()` request (ADR 0010 Links; A2/P-12: no multi-window support
        // in the MVP, so there is no legitimate target for one to open into).
        .on_new_window(|_url, _features| NewWindowResponse::Deny);

    builder = apply_title_bar_chrome(builder);

    builder.build()?;
    Ok(())
}

/// macOS: an overlay title bar with hidden title, so the traffic lights float over the app's own
/// content instead of a native bar (PoirotApp.swift:38-46's `.windowStyle(.hiddenTitleBar)`).
/// `TitleBarStyle::Overlay`'s own doc caveat is real and not fully addressable from this task
/// alone: it needs a `data-tauri-drag-region` element in the frontend to stay draggable, which
/// does not exist until M2-09 builds the real app shell -- recorded honestly in this task's own
/// progress-log entry rather than guessed at here.
#[cfg(target_os = "macos")]
fn apply_title_bar_chrome<R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: WebviewWindowBuilder<'_, R, M>,
) -> WebviewWindowBuilder<'_, R, M> {
    builder.title_bar_style(tauri::TitleBarStyle::Overlay).hidden_title(true)
}

/// Windows and Linux: native decorations, no overlay (port-plan P-4).
#[cfg(not(target_os = "macos"))]
fn apply_title_bar_chrome<R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: WebviewWindowBuilder<'_, R, M>,
) -> WebviewWindowBuilder<'_, R, M> {
    builder.decorations(true)
}
