//! MCP server status (Services/Config/MCPServerStatusChecker.swift). Poirot shelled out to
//! `/usr/bin/pgrep`; Heimdall reads the process list once through `sysinfo` and matches the same
//! patterns against each process's joined command line, on every OS.
//!
//! Order of checks: cloud integrations keep their status; a server in the auth cache needs auth;
//! plugin servers are running when their plugin name appears in a command line; HTTP and SSE
//! servers are "connected" when their URL is well formed (plan decision P-5: no network probe);
//! stdio servers are connected when their command pattern matches a running process.

use std::collections::BTreeSet;
use std::path::Path;

use regex::{Regex, RegexBuilder};

use crate::models::config::mcp::{McpServer, McpServerSource, McpServerStatus};
use crate::paths::TargetOs;

/// Every running process's command line, arguments joined by spaces. Processes whose command line
/// cannot be read (another user's, for example) are left out.
pub fn running_command_lines() -> Vec<String> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always),
    );
    sys.processes()
        .values()
        .map(|p| {
            p.cmd()
                .iter()
                .map(|a| a.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|line| !line.is_empty())
        .collect()
}

/// Sets every server's status. If any user stdio server is connected, Claude Code has started
/// them all, so a user stdio server still unknown most likely failed (MCPServerStatusChecker.swift:15-18).
pub fn resolve_statuses(servers: &mut [McpServer], auth: &BTreeSet<String>, processes: &[String], os: TargetOs) {
    for s in servers.iter_mut() {
        s.status = status(s, auth, processes, os);
    }
    let user_stdio = |s: &McpServer| s.source == McpServerSource::User && s.command.is_some();
    if servers
        .iter()
        .any(|s| user_stdio(s) && s.status == McpServerStatus::Connected)
    {
        for s in servers
            .iter_mut()
            .filter(|s| user_stdio(s) && s.status == McpServerStatus::Unknown)
        {
            s.status = McpServerStatus::Failed;
        }
    }
}

pub fn status(server: &McpServer, auth: &BTreeSet<String>, processes: &[String], os: TargetOs) -> McpServerStatus {
    if server.source == McpServerSource::CloudIntegration {
        return server.status;
    }
    if auth.contains(&server.raw_name) {
        return McpServerStatus::NeedsAuth;
    }
    if server.source == McpServerSource::Plugin {
        // `plugin:<name>:<server>`: running when `<name>` appears in a command line.
        let Some(name) = server.raw_name.split(':').nth(1) else {
            return McpServerStatus::Unknown;
        };
        return running(&[name.to_string()], processes, os);
    }
    let transport = server.transport.as_deref();
    if server.url.is_some() || matches!(transport, Some("http" | "sse")) {
        // P-5: a well-formed URL counts as connected; Heimdall does not probe the network.
        return match server.url.as_deref().map(url::Url::parse) {
            Some(Ok(_)) => McpServerStatus::Connected,
            _ => McpServerStatus::Unreachable,
        };
    }
    match &server.command {
        Some(command) => running(&process_pattern(command, &server.args), processes, os),
        None => McpServerStatus::Unknown,
    }
}

/// The parts a running server's command line must contain, in order (MCPServerStatusChecker.swift:160-214).
///
/// For `npx` and `uvx` the running process is usually `node …/_npx/…/node_modules/<package>/…`, not
/// the runner, so the package is matched: `@scope/name` as `scope` then `name`, an unscoped
/// package as the runner then the package. Version suffixes (`@latest`) are dropped. Other
/// commands match the executable followed by their non-flag arguments.
pub fn process_pattern(command: &str, args: &[String]) -> Vec<String> {
    let executable = Path::new(command)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(command)
        .to_string();
    // `npx.cmd` and `uvx.exe` on Windows are the same runners.
    let stem = Path::new(&executable)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&executable)
        .to_ascii_lowercase();
    let Some(package) = args.iter().find(|a| !a.starts_with('-')) else {
        return vec![executable];
    };
    if stem == "npx" || stem == "uvx" {
        let cleaned = match package.split_once('/') {
            Some((scope, rest)) => format!("{scope}/{}", rest.split('@').next().unwrap_or(rest)),
            None => package.split('@').next().unwrap_or(package).to_string(),
        };
        return match cleaned.strip_prefix('@').and_then(|c| c.split_once('/')) {
            Some((scope, name)) => vec![scope.to_string(), name.to_string()],
            None => vec![stem, cleaned],
        };
    }
    let significant: Vec<String> = args.iter().filter(|a| !a.starts_with('-')).cloned().collect();
    std::iter::once(executable)
        .chain(if significant.is_empty() {
            args.to_vec()
        } else {
            significant
        })
        .collect()
}

/// `part1.*part2.*…` with each part matched literally; case-insensitive on Windows, where paths are.
pub fn pattern_regex(parts: &[String], os: TargetOs) -> Option<Regex> {
    let joined = parts.iter().map(|p| regex::escape(p)).collect::<Vec<_>>().join(".*");
    RegexBuilder::new(&joined)
        .case_insensitive(os == TargetOs::Windows)
        .build()
        .ok()
}

fn running(parts: &[String], processes: &[String], os: TargetOs) -> McpServerStatus {
    match pattern_regex(parts, os) {
        Some(re) if processes.iter().any(|line| re.is_match(line)) => McpServerStatus::Connected,
        _ => McpServerStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::mcp::McpServerDefinition;
    use crate::models::config::scope::ConfigScope;

    fn server(
        name: &str,
        transport: Option<&str>,
        command: Option<&str>,
        args: &[&str],
        url: Option<&str>,
        source: McpServerSource,
    ) -> McpServer {
        let def = McpServerDefinition {
            transport: transport.map(String::from),
            command: command.map(String::from),
            args: Some(args.iter().map(|a| a.to_string()).collect()),
            url: url.map(String::from),
            ..Default::default()
        };
        McpServer::new(
            format!("global-{name}"),
            name,
            name,
            ConfigScope::Global,
            source,
            &def,
            vec![],
            false,
        )
    }
    fn user(name: &str, transport: Option<&str>, command: Option<&str>, args: &[&str], url: Option<&str>) -> McpServer {
        server(name, transport, command, args, url, McpServerSource::User)
    }
    fn auth(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }
    fn check(s: &McpServer, auth_names: &[&str]) -> McpServerStatus {
        status(s, &auth(auth_names), &[], TargetOs::Macos)
    }
    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // Ported from PoirotTests/Services/MCPServerStatusCheckerTests.swift (process list injected).
    #[test]
    fn auth_cache_detection() {
        assert_eq!(
            check(
                &user("sentry", Some("http"), None, &[], Some("https://sentry.io/mcp")),
                &["sentry", "github"]
            ),
            McpServerStatus::NeedsAuth
        );
    }
    #[test]
    fn server_not_in_auth_cache() {
        assert_ne!(
            check(
                &user("myserver", Some("http"), None, &[], Some("https://example.com/mcp")),
                &["sentry", "github"]
            ),
            McpServerStatus::NeedsAuth
        );
    }
    #[test]
    fn http_server_connected() {
        assert_eq!(
            check(
                &user("api", Some("http"), None, &[], Some("https://api.example.com/mcp")),
                &[]
            ),
            McpServerStatus::Connected
        );
    }
    #[test]
    fn http_server_no_url() {
        assert_eq!(
            check(&user("broken-http", Some("http"), None, &[], None), &[]),
            McpServerStatus::Unreachable
        );
    }
    #[test]
    fn sse_server_connected() {
        assert_eq!(
            check(
                &user(
                    "stream",
                    Some("sse"),
                    None,
                    &[],
                    Some("https://stream.example.com/events")
                ),
                &[]
            ),
            McpServerStatus::Connected
        );
    }
    #[test]
    fn url_inferred_http() {
        assert_eq!(
            check(
                &user("notion", None, None, &[], Some("https://mcp.notion.com/mcp")),
                &[]
            ),
            McpServerStatus::Connected
        );
    }
    #[test]
    fn stdio_no_command() {
        assert_eq!(
            check(&user("empty", Some("stdio"), None, &[], None), &[]),
            McpServerStatus::Unknown
        );
    }
    #[test]
    fn no_transport_info() {
        assert_eq!(
            check(&user("mystery", None, None, &[], None), &[]),
            McpServerStatus::Unknown
        );
    }
    #[test]
    fn bulk_resolution() {
        let mut servers = [
            user("http-ok", Some("http"), None, &[], Some("https://example.com/mcp")),
            user("unknown-server", None, None, &[], None),
            user(
                "auth-needed",
                Some("http"),
                None,
                &[],
                Some("https://auth.example.com/mcp"),
            ),
        ];
        resolve_statuses(&mut servers, &auth(&["auth-needed"]), &[], TargetOs::Macos);
        assert_eq!(
            servers.map(|s| s.status),
            [
                McpServerStatus::Connected,
                McpServerStatus::Unknown,
                McpServerStatus::NeedsAuth
            ]
        );
    }
    #[test]
    fn auth_cache_precedence() {
        assert_eq!(
            check(
                &user("github", Some("http"), None, &[], Some("https://api.github.com/mcp")),
                &["github"]
            ),
            McpServerStatus::NeedsAuth
        );
    }
    #[test]
    fn auth_cache_precedence_stdio() {
        assert_eq!(
            check(
                &user("my-stdio", Some("stdio"), Some("node"), &["server.js"], None),
                &["my-stdio"]
            ),
            McpServerStatus::NeedsAuth
        );
    }
    #[test]
    fn cloud_integration_preserves_status() {
        let mut s = server(
            "claude.ai Gmail",
            None,
            None,
            &[],
            None,
            McpServerSource::CloudIntegration,
        );
        s.status = McpServerStatus::NeedsAuth;
        assert_eq!(check(&s, &[]), McpServerStatus::NeedsAuth);
    }
    #[test]
    fn cloud_integration_ignores_auth_cache() {
        let mut s = server(
            "claude.ai Gmail",
            None,
            None,
            &[],
            None,
            McpServerSource::CloudIntegration,
        );
        s.status = McpServerStatus::NeedsAuth;
        assert_eq!(check(&s, &["some-other-server"]), McpServerStatus::NeedsAuth);
    }
    #[test]
    fn plugin_server_no_process() {
        assert_eq!(
            check(
                &server(
                    "plugin:nonexistent-plugin:nonexistent-plugin",
                    None,
                    None,
                    &[],
                    None,
                    McpServerSource::Plugin
                ),
                &[]
            ),
            McpServerStatus::Unknown
        );
    }

    #[test]
    fn patterns_follow_the_runner_rules() {
        let p = |cmd: &str, args: &[&str]| process_pattern(cmd, &strings(args));
        assert_eq!(
            p("npx", &["-y", "@modelcontextprotocol/server-filesystem"]),
            ["modelcontextprotocol", "server-filesystem"]
        );
        assert_eq!(p("npx", &["-y", "@playwright/mcp@latest"]), ["playwright", "mcp"]);
        assert_eq!(
            p("/usr/local/bin/uvx", &["perplexity-mcp@1.2"]),
            ["uvx", "perplexity-mcp"]
        );
        assert_eq!(
            p("node", &["server.js", "--port", "3000"]),
            ["node", "server.js", "3000"]
        );
        assert_eq!(p("python", &[]), ["python"]);
        assert_eq!(p("npx", &["--yes"]), ["npx"]);
    }

    #[test]
    fn windows_npx_cmd_matches_the_node_process_it_starts() {
        let s = user(
            "fs",
            Some("stdio"),
            Some(r"C:\Program Files\nodejs\npx.cmd"),
            &["-y", "@modelcontextprotocol/server-filesystem", r"C:\work"],
            None,
        );
        let processes = strings(&[
            r"C:\Windows\System32\conhost.exe 0x4",
            r#""C:\Program Files\nodejs\node.exe" C:\Users\me\AppData\Local\npm-cache\_npx\9f2c\node_modules\.bin\..\@ModelContextProtocol\server-filesystem\dist\index.js C:\work"#,
        ]);
        assert_eq!(
            process_pattern(s.command.as_deref().unwrap(), &s.args),
            ["modelcontextprotocol", "server-filesystem"]
        );
        assert_eq!(
            status(&s, &BTreeSet::new(), &processes, TargetOs::Windows),
            McpServerStatus::Connected
        );
        assert_eq!(
            status(&s, &BTreeSet::new(), &processes[..1], TargetOs::Windows),
            McpServerStatus::Unknown
        );
    }

    #[test]
    fn a_connected_stdio_server_marks_the_idle_ones_failed() {
        let mut servers = [
            user("running", Some("stdio"), Some("node"), &["alpha.js"], None),
            user("idle", Some("stdio"), Some("node"), &["beta.js"], None),
            user("remote", Some("http"), None, &[], Some("https://x.example/mcp")),
        ];
        resolve_statuses(
            &mut servers,
            &BTreeSet::new(),
            &strings(&["node /srv/alpha.js"]),
            TargetOs::Linux,
        );
        assert_eq!(
            servers.map(|s| s.status),
            [
                McpServerStatus::Connected,
                McpServerStatus::Failed,
                McpServerStatus::Connected
            ]
        );
    }

    #[test]
    fn literal_parts_do_not_act_as_regex() {
        // Poirot passed arguments to pgrep unescaped, so `server.js` also matched `serverXjs`.
        let s = user("x", Some("stdio"), Some("node"), &["server.js"], None);
        assert_eq!(
            status(&s, &BTreeSet::new(), &strings(&["node serverXjs"]), TargetOs::Linux),
            McpServerStatus::Unknown
        );
    }

    #[test]
    fn the_real_process_list_is_readable() {
        assert!(!running_command_lines().is_empty());
    }
}
