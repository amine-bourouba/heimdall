//! Executes launch actions computed by heimdall-core, per OS (ADR 0003). Each of
//! `windows.rs`/`macos.rs`/`linux.rs` is declared under its own matching `#[cfg(target_os)]`
//! below, so only the file for the host actually building ever gets compiled at all -- there is
//! no cross-OS stub to keep in sync, since the "no OS-specific work needed here" case
//! (macOS/Linux) is written as a real, if trivially small, module rather than left unwritten.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as os;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as os;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as os;

use heimdall_core::os_integration::launch::{Argv, Launch};
use heimdall_core::{AppError, Result};

/// Refuses a `.bat`/`.cmd` *script* as the spawned program itself (AC1; ADR 0010 point 7):
/// Windows argument passing to one of these goes through `cmd.exe`, reintroducing the
/// shell-metacharacter risk argv-only spawning exists to avoid. `heimdall_core::os_integration::
/// launch::editor_cli_argv` already refuses this for its own one call site (an editor CLI resolved
/// to a `.cmd` shim); this is the same check at the actual execution boundary, so it also covers
/// every *other* argv this module ever spawns, not only the editor path.
fn refuse_script_shim(program: &str) -> Result<()> {
    let ext = std::path::Path::new(program)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if ext == "bat" || ext == "cmd" {
        return Err(AppError::invalid_input(format!(
            "refusing to spawn a .{ext} script directly"
        )));
    }
    Ok(())
}

/// Spawns `argv.program argv.args...` detached: `std::process::Command::spawn` returns as soon as
/// the child exists, and simply never calling `.wait()` (or even keeping the returned `Child`
/// around) is what "detached" means here -- the OS process keeps running independently once
/// spawned, regardless of what this function's own caller does next (AC1). No shell is ever
/// involved: `Command::new`/`.args` pass the program and its arguments on as a real argument
/// array, never a string a shell would re-parse.
pub fn spawn_detached(argv: &Argv) -> Result<()> {
    refuse_script_shim(&argv.program)?;
    let mut command = std::process::Command::new(&argv.program);
    command.args(&argv.args);
    if let Some(cwd) = &argv.cwd {
        command.current_dir(cwd);
    }
    os::prepare(&mut command);
    command
        .spawn()
        .map_err(|e| AppError::io(format!("failed to launch {}: {e}", argv.program)))?;
    Ok(())
}

/// Runs a [`Launch`] -- either kind `heimdall_core::os_integration::launch`'s own builders return.
pub fn run(launch: &Launch) -> Result<()> {
    match launch {
        Launch::Argv(argv) => spawn_detached(argv),
        Launch::Url(url) => open_url(url),
    }
}

/// Opens `url` through the opener plugin's own Rust API (AC2) -- never the webview, never a shell.
pub fn open_url(url: &str) -> Result<()> {
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| AppError::io(e.to_string()))
}

/// Reveals `path` in the OS file manager (AC2). Confirmed by reading `tauri-plugin-opener`'s own
/// `reveal_item_in_dir.rs` directly rather than assuming: its Linux implementation already tries
/// `org.freedesktop.FileManager1` (a real "select this file" reveal) first and falls back to just
/// opening the item's parent folder when that D-Bus service isn't available -- exactly the
/// "Linux reveal degrades to opening the parent folder" this task's own AC2 names, already built
/// into the plugin itself. Nothing OS-specific needs writing here for that half of AC2 at all.
pub fn reveal(path: &std::path::Path) -> Result<()> {
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(|e| AppError::io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_bat_and_cmd_case_insensitively() {
        for bad in [r"C:\tools\code.cmd", r"C:\tools\code.CMD", r"C:\tools\run.bat"] {
            assert!(refuse_script_shim(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn allows_a_real_executable() {
        assert!(refuse_script_shim(r"C:\tools\code.exe").is_ok());
        assert!(refuse_script_shim("wt.exe").is_ok());
        assert!(refuse_script_shim("/usr/bin/code").is_ok());
    }

    /// Spawns a real, trivial, always-present OS command and confirms `spawn_detached` itself
    /// does not error -- the one part of this module safe to exercise for real without opening a
    /// real URL or file-manager window on a shared machine. OS-gated because the command that is
    /// both "instant" and "guaranteed present" differs per OS; `os::prepare`'s own real
    /// `CREATE_NO_WINDOW` effect (Windows) can't be observed from here, only that it does not
    /// itself cause the spawn to fail.
    #[cfg(target_os = "windows")]
    #[test]
    fn spawn_detached_runs_a_real_trivial_command() {
        let argv = Argv {
            program: "cmd.exe".to_string(),
            args: vec!["/c".to_string(), "exit".to_string(), "0".to_string()],
            cwd: None,
        };
        spawn_detached(&argv).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn spawn_detached_runs_a_real_trivial_command() {
        let argv = Argv {
            program: "/usr/bin/true".to_string(),
            args: vec![],
            cwd: None,
        };
        spawn_detached(&argv).unwrap();
    }

    #[test]
    fn spawn_detached_refuses_a_cmd_shim_before_ever_spawning() {
        let argv = Argv {
            program: r"C:\tools\code.cmd".to_string(),
            args: vec![],
            cwd: None,
        };
        assert!(matches!(
            spawn_detached(&argv).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }
}
