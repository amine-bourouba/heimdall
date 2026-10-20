//! PATH search for `claude` and editor CLIs (Views/Onboarding/OnboardingView.swift:281-297,
//! Utilities/EditorLauncher.swift:39-48), replacing the Swift reference app's four hardcoded macOS paths.
//!
//! Every function takes `PATH`, `PATHEXT`, the home directory and an `exists` check as plain
//! values, so the same code runs — and is tested — on every OS from any host (ADR 0003).
//!
//! | OS | Fallback directories tried after `PATH`, for `claude` only (ADR 0003) |
//! |---|---|
//! | Windows | `%APPDATA%\npm` |
//! | Linux | `~/.local/bin`, `~/.npm-global/bin` |
//! | macOS | `/opt/homebrew/bin`, `/usr/local/bin`, `~/.local/bin` |
//!
//! Editor CLIs are `PATH` only; ADR 0003 lists no extra fallback directories for them.

use std::path::{Path, PathBuf};

use crate::paths::TargetOs;

/// `PATH` splits on `;` on Windows and `:` elsewhere.
fn path_dirs(path_var: &str, os: TargetOs) -> impl Iterator<Item = &str> {
    let sep = if os == TargetOs::Windows { ';' } else { ':' };
    path_var.split(sep).filter(|s| !s.is_empty())
}

/// The file names a bare `name` resolves to: itself on Unix; itself plus one name per `PATHEXT`
/// extension on Windows (`CreateProcess`'s own search order), lower-cased to match how installers
/// actually name their shims (`claude.cmd`, not `claude.CMD`).
fn candidate_names(name: &str, pathext: &str, os: TargetOs) -> Vec<String> {
    if os != TargetOs::Windows {
        return vec![name.to_string()];
    }
    std::iter::once(name.to_string())
        .chain(
            pathext
                .split(';')
                .filter(|e| !e.is_empty())
                .map(|ext| format!("{name}{}", ext.to_lowercase())),
        )
        .collect()
}

/// Joins `dir` and `name` with the separator `os` uses, not the host's native one. `Path::join`
/// always uses the *host's* separator, which silently mixes `/` and `\` when testing a
/// `TargetOs` other than the host's own (exactly the cross-host CI run ADR 0003 asks every OS
/// branch to support) — confirmed by a real `ubuntu-22.04`/`macos-latest` CI failure on this
/// module's own Windows-path tests, which passed only on Windows before M1-59's gate review
/// actually ran CI cross-platform for the first time. `os` always matches the host in the real,
/// shipped app, so this changes nothing there — it only makes a *foreign* `TargetOs`'s behaviour
/// host-independent, which is what these functions' own doc comments already claimed.
fn join_for_os(dir: &Path, name: &str, os: TargetOs) -> PathBuf {
    let sep = if os == TargetOs::Windows { '\\' } else { '/' };
    let mut s = dir.to_string_lossy().into_owned();
    if !s.is_empty() && !s.ends_with(sep) {
        s.push(sep);
    }
    s.push_str(name);
    PathBuf::from(s)
}

fn search_dirs(
    dirs: impl Iterator<Item = PathBuf>,
    name: &str,
    pathext: &str,
    os: TargetOs,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let candidates = candidate_names(name, pathext, os);
    for dir in dirs {
        for candidate in &candidates {
            let path = join_for_os(&dir, candidate, os);
            if exists(&path) {
                return Some(path);
            }
        }
    }
    None
}

/// Finds `name` on `PATH`, trying each `PATH` directory in order.
pub fn find_on_path(
    name: &str,
    path_var: &str,
    pathext: &str,
    os: TargetOs,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    search_dirs(path_dirs(path_var, os).map(PathBuf::from), name, pathext, os, exists)
}

/// Directories checked for `claude` after `PATH` comes up empty (the table above).
pub fn claude_fallback_dirs(home: &Path, os: TargetOs) -> Vec<PathBuf> {
    let j = |dir: &Path, name: &str| join_for_os(dir, name, os);
    let j2 = |a: &str, b: &str| j(&j(home, a), b);
    match os {
        TargetOs::Windows => vec![j(&j2("AppData", "Roaming"), "npm")],
        TargetOs::Linux => vec![j2(".local", "bin"), j2(".npm-global", "bin")],
        TargetOs::Macos => vec![
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            j2(".local", "bin"),
        ],
    }
}

/// The `claude` CLI: `PATH` first, then the OS's fallback directories.
pub fn find_claude_cli(
    path_var: &str,
    pathext: &str,
    home: &Path,
    os: TargetOs,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    find_on_path("claude", path_var, pathext, os, exists).or_else(|| {
        search_dirs(
            claude_fallback_dirs(home, os).into_iter(),
            "claude",
            pathext,
            os,
            exists,
        )
    })
}

/// An editor's CLI command (`PreferredEditor::cli_command`) on `PATH`; no extra fallback
/// directories, per ADR 0003.
pub fn find_editor_cli(
    command: &str,
    path_var: &str,
    pathext: &str,
    os: TargetOs,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    find_on_path(command, path_var, pathext, os, exists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// An `exists` backed by an explicit set of paths, so no test touches the real filesystem.
    fn fs(paths: &[&str]) -> impl Fn(&Path) -> bool {
        let set: HashSet<PathBuf> = paths.iter().map(PathBuf::from).collect();
        move |p: &Path| set.contains(p)
    }
    const NO_PATHEXT: &str = "";
    const WIN_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD;.VBS;.JS;.WS;.MSC";

    #[test]
    fn finds_claude_on_windows_path_via_pathext() {
        let exists = fs(&[r"C:\tools\claude.cmd"]);
        let found = find_claude_cli(
            r"C:\Windows\System32;C:\tools",
            WIN_PATHEXT,
            Path::new(r"C:\Users\me"),
            TargetOs::Windows,
            &exists,
        );
        assert_eq!(found, Some(PathBuf::from(r"C:\tools\claude.cmd")));
    }
    #[test]
    fn finds_claude_on_windows_appdata_npm_fallback() {
        let exists = fs(&[r"C:\Users\me\AppData\Roaming\npm\claude.cmd"]);
        let found = find_claude_cli(
            r"C:\Windows\System32",
            WIN_PATHEXT,
            Path::new(r"C:\Users\me"),
            TargetOs::Windows,
            &exists,
        );
        assert_eq!(
            found,
            Some(PathBuf::from(r"C:\Users\me\AppData\Roaming\npm\claude.cmd"))
        );
    }
    #[test]
    fn finds_claude_on_linux_local_bin_fallback() {
        let exists = fs(&["/home/me/.local/bin/claude"]);
        let found = find_claude_cli(
            "/usr/bin:/bin",
            NO_PATHEXT,
            Path::new("/home/me"),
            TargetOs::Linux,
            &exists,
        );
        assert_eq!(found, Some(PathBuf::from("/home/me/.local/bin/claude")));
    }
    #[test]
    fn finds_claude_on_linux_npm_global_fallback() {
        let exists = fs(&["/home/me/.npm-global/bin/claude"]);
        let found = find_claude_cli("/usr/bin", NO_PATHEXT, Path::new("/home/me"), TargetOs::Linux, &exists);
        assert_eq!(found, Some(PathBuf::from("/home/me/.npm-global/bin/claude")));
    }
    #[test]
    fn finds_claude_on_macos_homebrew_fallback() {
        let exists = fs(&["/opt/homebrew/bin/claude"]);
        let found = find_claude_cli(
            "/usr/bin:/bin",
            NO_PATHEXT,
            Path::new("/Users/me"),
            TargetOs::Macos,
            &exists,
        );
        assert_eq!(found, Some(PathBuf::from("/opt/homebrew/bin/claude")));
    }
    #[test]
    fn prefers_path_over_any_fallback_directory() {
        let exists = fs(&["/usr/local/bin/claude", "/opt/homebrew/bin/claude"]);
        let found = find_claude_cli(
            "/usr/local/bin",
            NO_PATHEXT,
            Path::new("/Users/me"),
            TargetOs::Macos,
            &exists,
        );
        assert_eq!(found, Some(PathBuf::from("/usr/local/bin/claude")));
    }
    #[test]
    fn not_found_anywhere_is_none() {
        let exists = fs(&[]);
        assert!(find_claude_cli("/usr/bin", NO_PATHEXT, Path::new("/Users/me"), TargetOs::Macos, &exists).is_none());
    }
    #[test]
    fn empty_path_entries_are_skipped() {
        let exists = fs(&["/usr/local/bin/claude"]);
        let found = find_claude_cli(
            ":/usr/local/bin::",
            NO_PATHEXT,
            Path::new("/Users/me"),
            TargetOs::Macos,
            &exists,
        );
        assert_eq!(found, Some(PathBuf::from("/usr/local/bin/claude")));
    }

    #[test]
    fn finds_an_editor_cli_on_path_with_no_fallback_directories() {
        let exists = fs(&["/usr/local/bin/code"]);
        assert_eq!(
            find_editor_cli("code", "/usr/bin:/usr/local/bin", NO_PATHEXT, TargetOs::Macos, &exists),
            Some(PathBuf::from("/usr/local/bin/code"))
        );
        // A directory that would satisfy claude's own fallback rules does not help an editor.
        assert!(
            find_editor_cli(
                "code",
                "/usr/bin",
                NO_PATHEXT,
                TargetOs::Macos,
                &fs(&["/opt/homebrew/bin/code"])
            )
            .is_none()
        );
    }
    #[test]
    fn finds_vs_codes_windows_shim_via_pathext() {
        let exists = fs(&[r"C:\Users\me\AppData\Local\Programs\Microsoft VS Code\bin\code.cmd"]);
        let dir = r"C:\Users\me\AppData\Local\Programs\Microsoft VS Code\bin";
        assert_eq!(
            find_editor_cli("code", dir, WIN_PATHEXT, TargetOs::Windows, &exists),
            Some(PathBuf::from(format!("{dir}\\code.cmd")))
        );
    }
    #[test]
    fn bare_name_is_tried_before_pathext_extensions() {
        // Both a bare "zed" and "zed.cmd" exist; the bare name (matching CreateProcess's own
        // resolution order) wins.
        let exists = fs(&[r"C:\tools\zed", r"C:\tools\zed.cmd"]);
        assert_eq!(
            find_editor_cli("zed", r"C:\tools", WIN_PATHEXT, TargetOs::Windows, &exists),
            Some(PathBuf::from(r"C:\tools\zed"))
        );
    }

    #[test]
    fn fallback_dirs_match_the_adr_0003_table() {
        let home = Path::new("/home/me");
        assert_eq!(
            claude_fallback_dirs(home, TargetOs::Linux),
            [home.join(".local/bin"), home.join(".npm-global/bin")]
        );
        assert_eq!(
            claude_fallback_dirs(home, TargetOs::Macos),
            [
                PathBuf::from("/opt/homebrew/bin"),
                PathBuf::from("/usr/local/bin"),
                home.join(".local/bin")
            ]
        );
        assert_eq!(
            claude_fallback_dirs(Path::new(r"C:\Users\me"), TargetOs::Windows),
            [PathBuf::from(r"C:\Users\me\AppData\Roaming\npm")]
        );
    }
}
