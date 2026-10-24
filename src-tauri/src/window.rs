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

use tauri::Url;
use tauri::utils::config::WebviewUrl;
use tauri::webview::{NewWindowResponse, WebviewWindowBuilder};

const TITLE: &str = "Heimdall";
const WIDTH: f64 = 1280.0;
const HEIGHT: f64 = 820.0;
const MIN_WIDTH: f64 = 900.0;
const MIN_HEIGHT: f64 = 600.0;

/// `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` lets a debug build open WebView2's CDP debugging port
/// for `e2e/support/tauriCdp.ts` (M2-26/M2-29). Reading it in a release build would let an
/// attacker-controlled environment variable open a remote-debugging port on a shipped app -- full
/// control of the webview, cookies included (ADR 0010) -- so the release build never even
/// contains the code path that reads it, same convention as `state.rs`'s `HEIMDALL_CLAUDE_HOME`.
///
/// This cannot be left to WebView2Loader's own env-var fallback: confirmed directly (a real CI
/// run, with the env var correctly set before the process spawns, produced a fully working app --
/// real window, `Responding: True` -- with zero listening ports) and in wry 0.55.1's own source
/// (`webview2/mod.rs`'s `set_additional_browser_arguments` call runs unconditionally, substituting
/// wry's own default string whenever this builder method is never called) that the env var's
/// documented fallback behavior belongs to a different WebView2Loader entry point than the one wry
/// actually uses. wry's own default disables are preserved here since Tauri's own doc comment on
/// this method says they are lost otherwise: "By default wry passes
/// `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`".
#[cfg(debug_assertions)]
fn webview2_additional_browser_args() -> Option<String> {
    let cdp_flag = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").ok()?;
    if cdp_flag.is_empty() {
        return None;
    }
    Some(format!(
        "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection {cdp_flag}"
    ))
}

#[cfg(not(debug_assertions))]
fn webview2_additional_browser_args() -> Option<String> {
    None
}

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

    if let Some(args) = webview2_additional_browser_args() {
        builder = builder.additional_browser_args(&args);
    }

    builder.build()?;
    Ok(())
}

/// macOS: an overlay title bar with hidden title, so the traffic lights float over the app's own
/// content instead of a native bar (the Swift reference app's PoirotApp.swift:38-46's `.windowStyle(.hiddenTitleBar)`).
/// `TitleBarStyle::Overlay`'s own doc caveat is real and not fully addressable from this task
/// alone: it needs a `data-tauri-drag-region` element in the frontend to stay draggable, which
/// does not exist until M2-09 builds the real app shell -- recorded honestly in this task's own
/// progress-log entry rather than guessed at here.
#[cfg(target_os = "macos")]
fn apply_title_bar_chrome<R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: WebviewWindowBuilder<'_, R, M>,
) -> WebviewWindowBuilder<'_, R, M> {
    builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
}

/// Windows and Linux: native decorations, no overlay (port-plan P-4).
#[cfg(not(target_os = "macos"))]
fn apply_title_bar_chrome<R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: WebviewWindowBuilder<'_, R, M>,
) -> WebviewWindowBuilder<'_, R, M> {
    builder.decorations(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cdp_override_only_takes_effect_in_a_debug_build() {
        // Same reasoning as state.rs's equivalent test: `cfg!(debug_assertions)` reflects
        // whichever profile actually compiled this test binary, so `cargo test --release` is
        // what proves the release arm.
        unsafe { std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--remote-debugging-port=9223") };
        let found = webview2_additional_browser_args();
        unsafe { std::env::remove_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS") };
        if cfg!(debug_assertions) {
            assert_eq!(
                found,
                Some(
                    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --remote-debugging-port=9223"
                        .to_string()
                )
            );
        } else {
            assert_eq!(
                found, None,
                "a release build must never read WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"
            );
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn an_empty_cdp_override_is_treated_as_unset() {
        unsafe { std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "") };
        let found = webview2_additional_browser_args();
        unsafe { std::env::remove_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS") };
        assert_eq!(found, None);
    }
}
