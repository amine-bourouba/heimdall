//! Pure argv/URL builders for terminals and editors (Utilities/TerminalLauncher.swift,
//! Utilities/EditorLauncher.swift), plus the input validation ADR 0010 requires before any of them
//! run. Every builder returns data — a program with an argument array, or a URL string — never a
//! shell string, for `src-tauri/src/platform/*.rs` (M3-52) to spawn or open.
//!
//! Heimdall drops the Swift reference app's AppleScript "run this command" path (it had no callers) for one
//! behaviour on every OS: copy `claude --resume <id>` to the clipboard, then open the terminal at
//! the project folder with no command injected (port-plan C-5, ADR 0003).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use specta::Type;

use crate::error::{AppError, Result};
use crate::models::config::scope::ConfigRef;
use crate::models::prefs::{PreferredEditor, PreferredTerminal};
use crate::paths::{self, TargetOs};
use crate::projects::discovery::is_uuid;

/// What `open_in_editor` should open (contract `EditorTarget`): every variant names an id the
/// command resolves itself, never a path from the webview (ADR 0010, rule 5).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EditorTarget {
    #[serde(rename_all = "camelCase")]
    ToolBlock {
        project_id: String,
        session_id: String,
        block_id: String,
        line: Option<u32>,
    },
    #[serde(rename_all = "camelCase")]
    ConfigItem { config_ref: ConfigRef },
    #[serde(rename_all = "camelCase")]
    MemoryFile { source_id: String, file_id: String },
    /// Opens `~/.claude.json` at the named server's line.
    #[serde(rename_all = "camelCase")]
    McpConfig { server_name: String },
}

/// What `reveal_in_file_manager` should reveal (contract `RevealTarget`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RevealTarget {
    #[serde(rename_all = "camelCase")]
    Session {
        project_id: String,
        session_id: String,
    },
    #[serde(rename_all = "camelCase")]
    ConfigItem {
        config_ref: ConfigRef,
    },
    McpConfigFile,
}

/// A program and its arguments, never a shell string; `cwd` is a working directory to launch in,
/// not an argv element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argv {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

impl Argv {
    fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
        }
    }
    fn in_dir(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    Argv(Argv),
    Url(String),
}

/// Rejects a file target that is relative, a UNC path, starts with `-` (a program could read it as
/// a flag even though it arrives as its own argv element), or that the caller's own filesystem
/// check says is not a regular file.
pub fn validate_launch_path(path: &str, os: TargetOs, is_regular_file: bool) -> Result<()> {
    validate_target(path, os, is_regular_file, "a regular file")
}

/// The same checks as [`validate_launch_path`], for a terminal's working directory.
pub fn validate_cwd(path: &str, os: TargetOs, is_directory: bool) -> Result<()> {
    validate_target(path, os, is_directory, "a directory")
}

fn validate_target(path: &str, os: TargetOs, exists_as_expected: bool, expected: &str) -> Result<()> {
    if path.starts_with('-') {
        return Err(AppError::invalid_input(format!(
            "{path:?} looks like a command flag, not a path"
        )));
    }
    if !paths::is_absolute(path, os) {
        return Err(AppError::invalid_input(format!("{path:?} is not absolute")));
    }
    if paths::is_unc(path) {
        return Err(AppError::invalid_input(format!("{path:?} is a network path")));
    }
    if !exists_as_expected {
        return Err(AppError::invalid_input(format!("{path:?} is not {expected}")));
    }
    Ok(())
}

/// `^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$` or `^agent-[A-Za-z0-9_-]{1,128}$`
/// (docs/ipc-contract.md `SessionId`).
pub fn valid_session_id(id: &str) -> bool {
    match id.strip_prefix("agent-") {
        Some(rest) => {
            (1..=128).contains(&rest.len())
                && rest
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        }
        None => is_uuid(id),
    }
}

/// The clipboard text for "resume this session" (port-plan C-5), after validating the id.
pub fn resume_command_text(session_id: &str) -> Result<String> {
    if !valid_session_id(session_id) {
        return Err(AppError::invalid_input(format!(
            "{session_id:?} is not a valid session id"
        )));
    }
    Ok(format!("claude --resume {session_id}"))
}

/// Opens `terminal` at `cwd`. No command is ever injected — the resume command goes to the
/// clipboard instead ([`resume_command_text`]), because Windows Terminal's own command-line
/// parser splits on an unquoted `;` regardless of shell quoting, and every other terminal here
/// would otherwise need its own escaping rules for a string it never actually needs to receive.
pub fn open_terminal(terminal: PreferredTerminal, cwd: &str, os: TargetOs, is_directory: bool) -> Result<Launch> {
    validate_cwd(cwd, os, is_directory)?;
    use PreferredTerminal::{Alacritty, Cmd, GnomeTerminal, Kitty, Konsole, Powershell, System, WindowsTerminal};
    let argv = match (terminal, os) {
        (WindowsTerminal, TargetOs::Windows) => Argv::new("wt.exe", vec!["-d".into(), cwd.into()]),
        (Powershell, TargetOs::Windows) => Argv::new("powershell.exe", vec![]).in_dir(cwd),
        (Cmd, TargetOs::Windows) => Argv::new("cmd.exe", vec![]).in_dir(cwd),
        (GnomeTerminal, TargetOs::Linux) => Argv::new("gnome-terminal", vec![format!("--working-directory={cwd}")]),
        (Konsole, TargetOs::Linux) => Argv::new("konsole", vec!["--workdir".into(), cwd.into()]),
        (Kitty, TargetOs::Linux) => Argv::new("kitty", vec!["--directory".into(), cwd.into()]),
        (Alacritty, TargetOs::Linux) => Argv::new("alacritty", vec!["--working-directory".into(), cwd.into()]),
        (System, TargetOs::Linux) => Argv::new("x-terminal-emulator", vec![]).in_dir(cwd),
        (t, TargetOs::Macos) => match macos_app_name(t) {
            Some(app) => Argv::new("open", vec!["-a".into(), app.into(), cwd.into()]),
            None => return Err(AppError::invalid_input(format!("{t:?} has no macOS app"))),
        },
        (t, os) => return Err(AppError::invalid_input(format!("{t:?} is not available on {os:?}"))),
    };
    Ok(Launch::Argv(argv))
}

/// The `open -a` app name for a terminal offered on macOS; `None` for a Windows- or Linux-only one.
fn macos_app_name(terminal: PreferredTerminal) -> Option<&'static str> {
    use PreferredTerminal::{Alacritty, Ghostty, Iterm2, Kitty, Terminal, Warp};
    Some(match terminal {
        Terminal => "Terminal",
        Iterm2 => "iTerm",
        Warp => "Warp",
        Ghostty => "Ghostty",
        Kitty => "kitty",
        Alacritty => "Alacritty",
        _ => return None,
    })
}

/// `<scheme>://file/<percent-encoded abs path>[:<line>]` (ADR 0003; exact scheme and encoding
/// pending M3-54's confirmation against each editor's real handler). `None` for Xcode, which has
/// no URL scheme; it opens through [`editor_cli_argv`] only.
pub fn editor_url(
    editor: PreferredEditor,
    abs_path: &str,
    line: Option<u32>,
    os: TargetOs,
    is_regular_file: bool,
) -> Result<Option<String>> {
    if editor == PreferredEditor::Xcode {
        return Ok(None);
    }
    validate_launch_path(abs_path, os, is_regular_file)?;
    let scheme = match editor {
        PreferredEditor::Vscode => "vscode",
        PreferredEditor::Cursor => "cursor",
        PreferredEditor::Zed => "zed",
        PreferredEditor::Xcode => unreachable!("handled above"),
    };
    let encoded = to_url_path(abs_path);
    Ok(Some(match line {
        Some(n) => format!("{scheme}://file{encoded}:{n}"),
        None => format!("{scheme}://file{encoded}"),
    }))
}

/// The CLI fallback for when a URL scheme is not confirmed installed, and Xcode's only path
/// (`EditorLauncher.swift:8-16`). Refuses a `.cmd` shim as the program itself — Windows argument
/// passing to a `.cmd` goes through `cmd.exe`, which reintroduces the shell-metacharacter risk
/// argv-only launching exists to avoid; the caller should prefer [`editor_url`] there instead.
pub fn editor_cli_argv(
    editor: PreferredEditor,
    abs_path: &str,
    line: Option<u32>,
    cli_path: &Path,
    os: TargetOs,
    is_regular_file: bool,
) -> Result<Argv> {
    validate_launch_path(abs_path, os, is_regular_file)?;
    if cli_path.extension().is_some_and(|e| e.eq_ignore_ascii_case("cmd")) {
        return Err(AppError::invalid_input(
            "refusing to spawn a .cmd shim directly; use the editor's URL scheme instead",
        ));
    }
    let args = match (editor, line) {
        (PreferredEditor::Vscode | PreferredEditor::Cursor, Some(n)) => {
            vec!["--goto".into(), format!("{abs_path}:{n}")]
        }
        (PreferredEditor::Zed, Some(n)) => vec![format!("{abs_path}:{n}")],
        (PreferredEditor::Xcode, Some(n)) => vec!["--line".into(), n.to_string(), abs_path.into()],
        (_, None) => vec![abs_path.into()],
    };
    Ok(Argv::new(cli_path.display().to_string(), args))
}

/// A `file://`-style path: forward slashes, exactly one leading `/` (a Windows path gains one
/// before its drive letter, the `file:///C:/...` convention), then percent-encoded.
fn to_url_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    if !normalized.starts_with('/') {
        normalized = format!("/{normalized}");
    }
    percent_encode_path(&normalized)
}

/// Percent-encodes every byte but `A-Za-z0-9-_.~` and the path punctuation `/` and `:` (needed for
/// path separators and a Windows drive letter).
fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for b in path.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID: &str = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

    // --- validate_launch_path / validate_cwd ---------------------------------------------------
    #[test]
    fn rejects_a_relative_path() {
        assert!(validate_launch_path("src/main.rs", TargetOs::Linux, true).is_err());
    }
    #[test]
    fn rejects_a_leading_dash() {
        assert!(validate_launch_path("-rf", TargetOs::Linux, true).is_err());
        assert!(validate_cwd("-rf", TargetOs::Linux, true).is_err());
    }
    #[test]
    fn rejects_a_unc_path() {
        assert!(validate_launch_path(r"\\server\share\file.txt", TargetOs::Windows, true).is_err());
    }
    #[test]
    fn rejects_a_non_regular_file_or_non_directory() {
        assert!(validate_launch_path("/Users/me/file.txt", TargetOs::Macos, false).is_err());
        assert!(validate_cwd("/Users/me/project", TargetOs::Macos, false).is_err());
    }
    #[test]
    fn accepts_a_valid_absolute_target() {
        assert!(validate_launch_path(r"C:\Users\me\file.txt", TargetOs::Windows, true).is_ok());
        assert!(validate_cwd("/home/me/project", TargetOs::Linux, true).is_ok());
    }

    // --- session ids ----------------------------------------------------------------------------
    #[test]
    fn session_ids_accept_uuid_and_agent_forms() {
        assert!(valid_session_id(UUID) && valid_session_id("agent-a1b2_c3-D4"));
        for bad in [
            "A1B2C3D4-E5F6-7890-ABCD-EF1234567890",
            "not-a-uuid",
            "agent-",
            "agent-has a space",
            &format!("agent-{}", "x".repeat(129)),
        ] {
            assert!(!valid_session_id(bad), "{bad}");
        }
    }
    #[test]
    fn resume_command_text_validates_the_id() {
        assert_eq!(resume_command_text(UUID).unwrap(), format!("claude --resume {UUID}"));
        assert!(resume_command_text("; rm -rf /").is_err());
    }

    // --- terminals --------------------------------------------------------------------------------
    #[test]
    fn windows_terminals_use_the_documented_flags() {
        let cwd = r"C:\Users\me\proj";
        let wt = open_terminal(PreferredTerminal::WindowsTerminal, cwd, TargetOs::Windows, true).unwrap();
        assert_eq!(wt, Launch::Argv(Argv::new("wt.exe", vec!["-d".into(), cwd.into()])));
        let ps = open_terminal(PreferredTerminal::Powershell, cwd, TargetOs::Windows, true).unwrap();
        assert_eq!(ps, Launch::Argv(Argv::new("powershell.exe", vec![]).in_dir(cwd)));
        let cmd = open_terminal(PreferredTerminal::Cmd, cwd, TargetOs::Windows, true).unwrap();
        assert_eq!(cmd, Launch::Argv(Argv::new("cmd.exe", vec![]).in_dir(cwd)));
    }
    #[test]
    fn linux_terminals_use_the_documented_flags() {
        let cwd = "/home/me/proj";
        let cases = [
            (
                PreferredTerminal::GnomeTerminal,
                "gnome-terminal",
                vec![format!("--working-directory={cwd}")],
            ),
            (
                PreferredTerminal::Konsole,
                "konsole",
                vec!["--workdir".to_string(), cwd.into()],
            ),
            (
                PreferredTerminal::Kitty,
                "kitty",
                vec!["--directory".to_string(), cwd.into()],
            ),
            (
                PreferredTerminal::Alacritty,
                "alacritty",
                vec!["--working-directory".to_string(), cwd.into()],
            ),
        ];
        for (terminal, program, args) in cases {
            assert_eq!(
                open_terminal(terminal, cwd, TargetOs::Linux, true).unwrap(),
                Launch::Argv(Argv::new(program, args)),
                "{terminal:?}"
            );
        }
        assert_eq!(
            open_terminal(PreferredTerminal::System, cwd, TargetOs::Linux, true).unwrap(),
            Launch::Argv(Argv::new("x-terminal-emulator", vec![]).in_dir(cwd))
        );
    }
    #[test]
    fn macos_terminals_all_go_through_open_dash_a() {
        let cwd = "/Users/me/proj";
        let cases = [
            (PreferredTerminal::Terminal, "Terminal"),
            (PreferredTerminal::Iterm2, "iTerm"),
            (PreferredTerminal::Warp, "Warp"),
            (PreferredTerminal::Ghostty, "Ghostty"),
            (PreferredTerminal::Kitty, "kitty"),
            (PreferredTerminal::Alacritty, "Alacritty"),
        ];
        for (terminal, app) in cases {
            let got = open_terminal(terminal, cwd, TargetOs::Macos, true).unwrap();
            assert_eq!(
                got,
                Launch::Argv(Argv::new("open", vec!["-a".into(), app.into(), cwd.into()])),
                "{terminal:?}"
            );
        }
    }
    #[test]
    fn a_terminal_not_offered_on_the_given_os_is_rejected() {
        assert!(open_terminal(PreferredTerminal::WindowsTerminal, "/home/me", TargetOs::Linux, true).is_err());
        assert!(
            open_terminal(
                PreferredTerminal::GnomeTerminal,
                r"C:\Users\me",
                TargetOs::Windows,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn an_invalid_cwd_is_rejected_before_any_terminal_specific_logic() {
        assert!(open_terminal(PreferredTerminal::Terminal, "relative/path", TargetOs::Macos, true).is_err());
        assert!(open_terminal(PreferredTerminal::Terminal, "-x", TargetOs::Macos, true).is_err());
    }

    #[test]
    fn windows_terminal_never_lets_a_semicolon_split_the_command() {
        // wt.exe's own parser splits an *unquoted* `;` into a new command; our builder never joins
        // args into one string, so a `;` inside the cwd stays inside its single argv element,
        // where normal Windows argv quoting (outside this module's scope) keeps it literal.
        let cwd = r"C:\Users\me\weird;name";
        let Launch::Argv(argv) =
            open_terminal(PreferredTerminal::WindowsTerminal, cwd, TargetOs::Windows, true).unwrap()
        else {
            panic!()
        };
        assert_eq!(argv.args, vec!["-d".to_string(), cwd.to_string()]);
        assert_eq!(argv.args.len(), 2);
    }

    /// the Swift reference app's own examples of characters a shell would treat specially; this table proves every
    /// terminal on every OS carries such a cwd through as one intact argv element (or `Command`'s
    /// working directory), never splitting or otherwise re-interpreting it — the safety property
    /// that using argv instead of a shell string is meant to provide.
    #[test]
    fn dangerous_characters_survive_as_one_intact_argv_element_on_every_os() {
        let dangerous = [r"C:\Users\me\a b", r#"C:\Users\me\a;b&c|d%e^f""#];
        for cwd in dangerous {
            let Launch::Argv(argv) =
                open_terminal(PreferredTerminal::WindowsTerminal, cwd, TargetOs::Windows, true).unwrap()
            else {
                panic!()
            };
            assert_eq!(argv.args.last(), Some(&cwd.to_string()));
        }
        for cwd in ["/home/me/a b", "/home/me/a;b&c|d%e^f\""] {
            let Launch::Argv(argv) = open_terminal(PreferredTerminal::Alacritty, cwd, TargetOs::Linux, true).unwrap()
            else {
                panic!()
            };
            assert_eq!(argv.args.last(), Some(&cwd.to_string()));
            let Launch::Argv(argv) = open_terminal(PreferredTerminal::Alacritty, cwd, TargetOs::Macos, true).unwrap()
            else {
                panic!()
            };
            assert_eq!(argv.args.last(), Some(&cwd.to_string()));
        }
    }

    // --- editors -------------------------------------------------------------------------------
    #[test]
    fn editor_urls_use_the_documented_scheme_with_and_without_a_line() {
        let path = "/Users/me/src/main.rs";
        assert_eq!(
            editor_url(PreferredEditor::Vscode, path, Some(42), TargetOs::Macos, true)
                .unwrap()
                .as_deref(),
            Some("vscode://file/Users/me/src/main.rs:42")
        );
        assert_eq!(
            editor_url(PreferredEditor::Cursor, path, None, TargetOs::Macos, true)
                .unwrap()
                .as_deref(),
            Some("cursor://file/Users/me/src/main.rs")
        );
        assert_eq!(
            editor_url(PreferredEditor::Zed, path, Some(7), TargetOs::Macos, true)
                .unwrap()
                .as_deref(),
            Some("zed://file/Users/me/src/main.rs:7")
        );
    }
    #[test]
    fn xcode_has_no_url_scheme() {
        assert_eq!(
            editor_url(
                PreferredEditor::Xcode,
                "/Users/me/main.swift",
                Some(1),
                TargetOs::Macos,
                true
            )
            .unwrap(),
            None
        );
    }
    #[test]
    fn editor_url_normalises_a_windows_path_and_percent_encodes_unsafe_characters() {
        let unix = editor_url(PreferredEditor::Vscode, "/Users/me/a b#c", None, TargetOs::Macos, true)
            .unwrap()
            .unwrap();
        assert_eq!(unix, "vscode://file/Users/me/a%20b%23c");
        let windows = editor_url(
            PreferredEditor::Vscode,
            r"C:\Users\me\a b.rs",
            Some(3),
            TargetOs::Windows,
            true,
        )
        .unwrap()
        .unwrap();
        assert_eq!(windows, "vscode://file/C:/Users/me/a%20b.rs:3");
    }
    #[test]
    fn editor_url_rejects_an_invalid_path() {
        assert!(editor_url(PreferredEditor::Vscode, "relative.rs", None, TargetOs::Linux, true).is_err());
        assert!(
            editor_url(
                PreferredEditor::Vscode,
                "/Users/me/gone.rs",
                None,
                TargetOs::Macos,
                false
            )
            .is_err()
        );
    }

    #[test]
    fn editor_cli_argv_matches_each_editors_shape() {
        let path = "/Users/me/main.rs";
        let cli = Path::new("/usr/local/bin/code");
        let with_line = editor_cli_argv(PreferredEditor::Vscode, path, Some(9), cli, TargetOs::Macos, true).unwrap();
        assert_eq!(
            with_line,
            Argv::new("/usr/local/bin/code", vec!["--goto".into(), format!("{path}:9")])
        );
        let zed = editor_cli_argv(
            PreferredEditor::Zed,
            path,
            Some(9),
            Path::new("/usr/local/bin/zed"),
            TargetOs::Macos,
            true,
        )
        .unwrap();
        assert_eq!(zed.args, vec![format!("{path}:9")]);
        let xed = editor_cli_argv(
            PreferredEditor::Xcode,
            path,
            Some(3),
            Path::new("/usr/bin/xed"),
            TargetOs::Macos,
            true,
        )
        .unwrap();
        assert_eq!(xed.args, vec!["--line".to_string(), "3".to_string(), path.to_string()]);
        let no_line = editor_cli_argv(
            PreferredEditor::Cursor,
            path,
            None,
            Path::new("/usr/local/bin/cursor"),
            TargetOs::Macos,
            true,
        )
        .unwrap();
        assert_eq!(no_line.args, vec![path.to_string()]);
    }
    #[test]
    fn editor_cli_argv_never_spawns_a_cmd_shim() {
        let err = editor_cli_argv(
            PreferredEditor::Vscode,
            r"C:\Users\me\main.rs",
            Some(1),
            Path::new(r"C:\tools\code.cmd"),
            TargetOs::Windows,
            true,
        );
        assert!(err.is_err());
        // A non-.cmd resolution (an .exe wrapper, or a bare name found without an extension) is fine.
        assert!(
            editor_cli_argv(
                PreferredEditor::Vscode,
                r"C:\Users\me\main.rs",
                Some(1),
                Path::new(r"C:\tools\code.exe"),
                TargetOs::Windows,
                true
            )
            .is_ok()
        );
    }
    #[test]
    fn editor_cli_argv_rejects_an_invalid_path() {
        assert!(
            editor_cli_argv(
                PreferredEditor::Vscode,
                "relative.rs",
                None,
                Path::new("/usr/local/bin/code"),
                TargetOs::Linux,
                true
            )
            .is_err()
        );
    }
}
