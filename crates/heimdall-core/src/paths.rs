//! Claude home resolution, path normalisation and per-OS comparison keys (ADR 0003).
//!
//! Every function that depends on the operating system takes a [`TargetOs`], so all three
//! variants are tested on any host. Normalised forms and comparison keys are for comparing
//! paths only; the original string is always what the UI displays.

use std::path::{Path, PathBuf};

use serde::Serialize;
use specta::Type;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TargetOs {
    Windows,
    Linux,
    Macos,
}

impl TargetOs {
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Linux
        }
    }

    /// Default file systems: NTFS and APFS compare names case-insensitively; ext4 does not.
    pub const fn case_insensitive(self) -> bool {
        matches!(self, Self::Windows | Self::Macos)
    }
}

/// Where Claude Code keeps its data: `root` is `~/.claude`, `config_file` is `~/.claude.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeHome {
    root: PathBuf,
    config_file: PathBuf,
}

impl ClaudeHome {
    /// The layout Claude Code uses by default for a user whose home folder is `user_home`.
    pub fn for_user_home(user_home: impl AsRef<Path>) -> Self {
        let home = user_home.as_ref();
        Self { root: home.join(".claude"), config_file: home.join(".claude.json") }
    }

    /// Explicit roots, used by tests and fixture trees.
    pub fn new(root: impl Into<PathBuf>, config_file: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), config_file: config_file.into() }
    }

    /// Resolve from the environment. Claude Code 2.1.282 treats `CLAUDE_CONFIG_DIR` as its
    /// "configuration home" and rejects a relative value (strings in its binary; progress log M1-06).
    /// With the override, `.claude.json` is assumed to live inside that folder (assumption A-2).
    pub fn from_env() -> Result<Self> {
        Self::resolve(std::env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from), user_home())
    }

    fn resolve(config_dir: Option<PathBuf>, user_home: Option<PathBuf>) -> Result<Self> {
        match config_dir.filter(|d| !d.as_os_str().is_empty()) {
            Some(dir) if dir.is_absolute() => Ok(Self { config_file: dir.join(".claude.json"), root: dir }),
            Some(dir) => Err(AppError::invalid_input("CLAUDE_CONFIG_DIR must be an absolute path").with_path(dir)),
            None => user_home
                .map(Self::for_user_home)
                .ok_or_else(|| AppError::not_found("the user's home folder could not be determined")),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }
    pub fn projects_dir(&self) -> PathBuf {
        self.root.join("projects")
    }
    pub fn history_file(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }
    pub fn todos_dir(&self) -> PathBuf {
        self.root.join("todos")
    }
    pub fn debug_dir(&self) -> PathBuf {
        self.root.join("debug")
    }
    pub fn file_history_dir(&self) -> PathBuf {
        self.root.join("file-history")
    }
    pub fn facets_dir(&self) -> PathBuf {
        self.root.join("usage-data").join("facets")
    }
    pub fn stats_cache_file(&self) -> PathBuf {
        self.root.join("stats-cache.json")
    }
    pub fn commands_dir(&self) -> PathBuf {
        self.root.join("commands")
    }
    pub fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }
    pub fn plans_dir(&self) -> PathBuf {
        self.root.join("plans")
    }
    pub fn output_styles_dir(&self) -> PathBuf {
        self.root.join("output-styles")
    }
    pub fn agents_dir(&self) -> PathBuf {
        self.root.join("agents")
    }
    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }
    pub fn installed_plugins_file(&self) -> PathBuf {
        self.root.join("plugins").join("installed_plugins.json")
    }
    pub fn mcp_auth_cache_file(&self) -> PathBuf {
        self.root.join("mcp-needs-auth-cache.json")
    }
}

/// The current user's home folder, however this OS finds it — the one place `dirs::home_dir` is
/// called, so every other caller in the workspace (including src-tauri) goes through this instead
/// of adding its own dependency on the `dirs` crate.
pub fn user_home() -> Option<PathBuf> {
    dirs::home_dir()
}

/// Canonical string form for comparison: `/` separators, no `\\?\` prefix, no repeated or trailing
/// separators, upper-case drive letter. UNC paths keep their leading `//`.
pub fn normalize(path: &str) -> String {
    let mut s = path.replace('\\', "/");
    if let Some(rest) = s.strip_prefix("//?/UNC/") {
        s = format!("//{rest}");
    } else if let Some(rest) = s.strip_prefix("//?/") {
        s = rest.to_string();
    }
    let unc = s.starts_with("//");
    let mut out = String::with_capacity(s.len());
    let mut prev_slash = false;
    for ch in s.chars() {
        if ch == '/' {
            if !prev_slash {
                out.push('/');
            }
            prev_slash = true;
        } else {
            out.push(ch);
            prev_slash = false;
        }
    }
    if unc {
        out.insert(0, '/');
    }
    let bytes = out.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        out.replace_range(0..1, &out[0..1].to_ascii_uppercase());
    }
    while out.len() > 1 && out.ends_with('/') && !(out.len() == 3 && out.as_bytes()[1] == b':') {
        out.pop();
    }
    out
}

/// Key for equality checks: normalised, and lower-cased where the default file system ignores case.
pub fn compare_key(path: &str, os: TargetOs) -> String {
    let n = normalize(path);
    if os.case_insensitive() { n.to_lowercase() } else { n }
}

pub fn same_path(a: &str, b: &str, os: TargetOs) -> bool {
    compare_key(a, os) == compare_key(b, os)
}

/// Port of `FileHistoryLoader.sameFile` (FileHistoryLoader.swift:173-178): two transcript paths refer
/// to the same file when one ends with the other on a segment boundary (`src/a.rs` versus
/// `C:\proj\src\a.rs`). Works across Windows and Unix separators.
pub fn same_file(lhs: &str, rhs: &str, os: TargetOs) -> bool {
    let with_root = |p: &str| {
        let k = compare_key(p, os);
        if k.starts_with('/') { k } else { format!("/{k}") }
    };
    let (l, r) = (with_root(lhs), with_root(rhs));
    l == r || l.ends_with(&r) || r.ends_with(&l)
}

/// `\\server\share`, `//server/share` and `\\?\UNC\server\share` are network paths.
pub fn is_unc(path: &str) -> bool {
    let n = normalize(path);
    n.starts_with("//")
}

pub fn is_absolute(path: &str, os: TargetOs) -> bool {
    let n = normalize(path);
    match os {
        TargetOs::Windows => {
            let b = n.as_bytes();
            n.starts_with("//") || (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'/')
        }
        TargetOs::Linux | TargetOs::Macos => n.starts_with('/'),
    }
}

/// Claude Code's project folder name: every character that is not an ASCII letter or digit
/// becomes `-` (`C:\Users\me\app` → `C--Users-me-app`, `/Users/me/app` → `-Users-me-app`).
pub fn encode_project_dir_name(path: &str) -> String {
    path.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect()
}

/// True when `path`, encoded, is the project folder `dir_name`. Compared case-insensitively
/// because Claude Code writes both `C:` and `c:` drive letters (fixtures/windows/MANIFEST.md).
pub fn encodes_to(path: &str, dir_name: &str) -> bool {
    encode_project_dir_name(path).eq_ignore_ascii_case(dir_name)
}

/// Last-resort reconstruction of a project path from its folder name, porting
/// `SessionLoader.decodeProjectPath` (SessionLoader.swift:314-353): at each step take the longest
/// hyphen-joined run of tokens that exists as a folder. `exists` answers "is this a folder?".
/// Returns the reconstructed absolute path; the display name is its last component.
pub fn decode_project_dir_name(encoded: &str, os: TargetOs, exists: impl Fn(&str) -> bool) -> String {
    let (mut resolved, rest) = match os {
        TargetOs::Windows => match windows_drive_prefix(encoded) {
            Some((drive, rest)) => (format!("{drive}:"), rest),
            None => (String::new(), encoded.trim_start_matches('-')),
        },
        TargetOs::Linux | TargetOs::Macos => (String::new(), encoded.strip_prefix('-').unwrap_or(encoded)),
    };
    let sep = if os == TargetOs::Windows { '\\' } else { '/' };
    let parts: Vec<&str> = rest.split('-').collect();
    if parts.iter().all(|p| p.is_empty()) {
        return encoded.to_string();
    }
    let mut i = 0;
    while i < parts.len() {
        let mut best = None;
        for j in i..parts.len() {
            let candidate = format!("{resolved}{sep}{}", parts[i..=j].join("-"));
            if exists(&candidate) {
                best = Some(j);
            }
        }
        match best {
            Some(j) => {
                resolved = format!("{resolved}{sep}{}", parts[i..=j].join("-"));
                i = j + 1;
            }
            None => {
                resolved = format!("{resolved}{sep}{}", parts[i..].join("-"));
                break;
            }
        }
    }
    resolved
}

/// `C--Users-...` → (`C`, `Users-...`).
fn windows_drive_prefix(encoded: &str) -> Option<(char, &str)> {
    let mut chars = encoded.chars();
    let drive = chars.next().filter(|c| c.is_ascii_alphabetic())?;
    encoded.get(1..3).filter(|s| *s == "--").map(|_| (drive.to_ascii_uppercase(), &encoded[3..]))
}

/// The last component of a path in either separator style.
pub fn last_component(path: &str) -> &str {
    path.trim_end_matches(['/', '\\']).rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Accepts `name` only if it can be joined to a folder as a single file name: not empty, not `.`
/// or `..`, and free of separators, drive colons and NUL. Session ids reach file paths this way.
pub fn plain_file_name(name: &str) -> Result<&str> {
    let bad = name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', ':', '\0']);
    if bad { Err(AppError::invalid_input(format!("not a plain file name: {name:?}"))) } else { Ok(name) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TargetOs::*;

    #[test]
    fn normalises_windows_and_unix_forms() {
        assert_eq!(normalize(r"C:\Users\a\x"), "C:/Users/a/x");
        assert_eq!(normalize("c:/Users/a/x"), "C:/Users/a/x");
        assert_eq!(normalize(r"\\?\C:\Users\a\x"), "C:/Users/a/x");
        assert_eq!(normalize(r"C:\Users//a\\x\"), "C:/Users/a/x");
        assert_eq!(normalize("/Users/a/x/"), "/Users/a/x");
        assert_eq!(normalize("C:\\"), "C:/");
        assert_eq!(normalize("/"), "/");
        assert_eq!(normalize(r"\\server\share\dir"), "//server/share/dir");
        assert_eq!(normalize(r"\\?\UNC\server\share"), "//server/share");
    }

    #[test]
    fn comparison_keys_follow_each_os_case_rules() {
        assert!(same_path(r"C:\Users\A\App", "c:/users/a/app", Windows));
        assert!(same_path("/Users/A/App", "/users/a/app", Macos));
        assert!(!same_path("/home/A/App", "/home/a/app", Linux));
        assert!(same_path("/home/a/app/", "/home/a/app", Linux));
    }

    #[test]
    fn same_file_matches_suffixes_on_segment_boundaries() {
        assert!(same_file("src/a.rs", r"C:\proj\src\a.rs", Windows));
        assert!(same_file("/Users/me/proj/src/a.swift", "src/a.swift", Macos));
        assert!(same_file("x.rs", "x.rs", Linux));
        assert!(!same_file("b.rs", "/proj/src/ab.rs", Linux), "suffix must start at a separator");
        assert!(!same_file("Src/A.rs", "/p/src/a.rs", Linux));
        assert!(same_file("Src/A.rs", "/p/src/a.rs", Windows));
    }

    #[test]
    fn detects_unc_and_absolute_paths() {
        assert!(is_unc(r"\\server\share"));
        assert!(is_unc("//server/share"));
        assert!(!is_unc(r"\\?\C:\x"));
        assert!(!is_unc(r"C:\x"));
        assert!(is_absolute(r"C:\x", Windows));
        assert!(is_absolute(r"\\server\share", Windows));
        assert!(!is_absolute(r"x\y", Windows));
        assert!(is_absolute("/x", Linux));
        assert!(!is_absolute("x", Macos));
    }

    #[test]
    fn encodes_project_folder_names_like_claude_code() {
        assert_eq!(encode_project_dir_name(r"C:\Users\me\Desktop\tech"), "C--Users-me-Desktop-tech");
        assert_eq!(encode_project_dir_name("/Users/me/src/my app"), "-Users-me-src-my-app");
        assert_eq!(encode_project_dir_name(r"C:\Users\me\FreePro - SI - test"), "C--Users-me-FreePro---SI---test");
        assert!(encodes_to(r"c:\Users\fixture\Desktop\tech\proj-11", "C--Users-fixture-Desktop-tech-proj-11"));
    }

    #[test]
    fn decodes_folder_names_by_probing_like_poirot() {
        let dirs = ["/Users", "/Users/me", "/Users/me/my-app"];
        let exists = |p: &str| dirs.contains(&p);
        assert_eq!(decode_project_dir_name("-Users-me-my-app", Macos, exists), "/Users/me/my-app");
        // A missing tail keeps its hyphens joined, as SessionLoader.swift:341-346 does.
        assert_eq!(decode_project_dir_name("-Users-me-gone-app", Macos, exists), "/Users/me/gone-app");
        let win = [r"C:\Users", r"C:\Users\pc", r"C:\Users\pc\Desktop"];
        let wexists = |p: &str| win.contains(&p);
        assert_eq!(decode_project_dir_name("c--Users-pc-Desktop-tech", Windows, wexists), r"C:\Users\pc\Desktop\tech");
        assert_eq!(last_component(r"C:\Users\pc\Desktop\tech"), "tech");
        assert_eq!(last_component("/Users/me/my-app/"), "my-app");
    }

    #[test]
    fn claude_home_resolution_honours_an_absolute_config_dir_only() {
        let home = ClaudeHome::resolve(None, Some(PathBuf::from("/home/u"))).unwrap();
        assert_eq!(home.root(), Path::new("/home/u/.claude"));
        assert_eq!(home.config_file(), Path::new("/home/u/.claude.json"));
        assert_eq!(home.projects_dir(), Path::new("/home/u/.claude/projects"));

        let abs = if cfg!(windows) { r"C:\cfg" } else { "/cfg" };
        let over = ClaudeHome::resolve(Some(PathBuf::from(abs)), Some(PathBuf::from("/home/u"))).unwrap();
        assert_eq!(over.root(), Path::new(abs));
        assert_eq!(over.config_file(), Path::new(abs).join(".claude.json"));

        assert!(matches!(
            ClaudeHome::resolve(Some(PathBuf::from("relative/cfg")), None),
            Err(AppError::InvalidInput { .. })
        ));
        assert!(ClaudeHome::resolve(Some(PathBuf::new()), Some(PathBuf::from("/h"))).is_ok(), "empty override is ignored");
        assert!(matches!(ClaudeHome::resolve(None, None), Err(AppError::NotFound { .. })));
    }

    #[test]
    fn plain_file_names_reject_anything_that_leaves_the_folder() {
        assert!(plain_file_name("0a1b-session").is_ok());
        for bad in ["", ".", "..", "../x", "a/b", "a\\b", "C:x", "a\0b"] {
            assert!(plain_file_name(bad).is_err(), "{bad:?}");
        }
    }
}
