//! Curated MCP server catalog (Models/Config/MCPServerCatalog.swift). Icons are Poirot's SF Symbol
//! names; the webview maps them to Lucide (M2-04 icon map).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum McpCatalogCategory {
    Developer,
    Productivity,
    Data,
    Filesystem,
}

impl McpCatalogCategory {
    pub const ALL: [Self; 4] = [Self::Developer, Self::Productivity, Self::Data, Self::Filesystem];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Developer => "Developer Tools",
            Self::Productivity => "Productivity",
            Self::Data => "Data & Search",
            Self::Filesystem => "Filesystem",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::Developer => "hammer",
            Self::Productivity => "tray.full",
            Self::Data => "magnifyingglass.circle",
            Self::Filesystem => "folder",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct McpEnvKey {
    pub id: String,
    pub label: String,
    pub description: String,
    pub placeholder: String,
    pub is_required: bool,
    pub is_sensitive: bool,
}

impl McpEnvKey {
    /// Defaults as in Swift: required, not sensitive, empty placeholder.
    pub fn new(id: &str, label: &str, description: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
            placeholder: String::new(),
            is_required: true,
            is_sensitive: false,
        }
    }
    fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.into();
        self
    }
    fn optional(mut self) -> Self {
        self.is_required = false;
        self
    }
    fn sensitive(mut self) -> Self {
        self.is_sensitive = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct McpCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: McpCatalogCategory,
    pub command: String,
    pub args: Vec<String>,
    pub env_keys: Vec<McpEnvKey>,
}

impl McpCatalogEntry {
    pub fn is_npx(&self) -> bool {
        self.command == "npx"
    }
}

/// The 15 bundled entries, in Poirot's order.
pub fn entries() -> Vec<McpCatalogEntry> {
    use McpCatalogCategory::*;
    let npx =
        |id: &str, name: &str, description: &str, icon: &str, category, package: &str, env_keys| McpCatalogEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            icon: icon.into(),
            category,
            command: "npx".into(),
            args: vec!["-y".into(), package.into()],
            env_keys,
        };
    let key = McpEnvKey::new;
    vec![
        npx(
            "github",
            "GitHub",
            "Interact with GitHub repositories, issues, pull requests, and more.",
            "arrow.triangle.branch",
            Developer,
            "@modelcontextprotocol/server-github",
            vec![
                key(
                    "GITHUB_PERSONAL_ACCESS_TOKEN",
                    "Personal Access Token",
                    "GitHub PAT with repo access",
                )
                .placeholder("ghp_...")
                .sensitive(),
            ],
        ),
        npx(
            "playwright",
            "Playwright",
            "Browser automation for testing and web scraping.",
            "theatermasks",
            Developer,
            "@anthropic-ai/mcp-server-playwright",
            vec![],
        ),
        npx(
            "puppeteer",
            "Puppeteer",
            "Headless Chrome browser automation and screenshots.",
            "globe",
            Developer,
            "@modelcontextprotocol/server-puppeteer",
            vec![],
        ),
        npx(
            "sentry",
            "Sentry",
            "Access Sentry issues, events, and error tracking data.",
            "ladybug",
            Developer,
            "@sentry/mcp-server",
            vec![
                key("SENTRY_AUTH_TOKEN", "Auth Token", "Sentry authentication token")
                    .placeholder("sntrys_...")
                    .sensitive(),
                key("SENTRY_ORG", "Organization", "Sentry organization slug")
                    .placeholder("my-org")
                    .optional(),
            ],
        ),
        npx(
            "linear",
            "Linear",
            "Manage Linear issues, projects, and workflows.",
            "list.bullet.rectangle",
            Developer,
            "@anthropic-ai/mcp-server-linear",
            vec![
                key("LINEAR_API_KEY", "API Key", "Linear personal API key")
                    .placeholder("lin_api_...")
                    .sensitive(),
            ],
        ),
        npx(
            "slack",
            "Slack",
            "Read and send Slack messages, manage channels.",
            "number",
            Productivity,
            "@modelcontextprotocol/server-slack",
            vec![
                key("SLACK_BOT_TOKEN", "Bot Token", "Slack bot OAuth token")
                    .placeholder("xoxb-...")
                    .sensitive(),
                key("SLACK_TEAM_ID", "Team ID", "Slack workspace team ID").placeholder("T0123456789"),
            ],
        ),
        npx(
            "notion",
            "Notion",
            "Search, read, and create Notion pages and databases.",
            "doc.richtext",
            Productivity,
            "@notionhq/notion-mcp-server",
            vec![
                key(
                    "OPENAPI_MCP_HEADERS",
                    "Authorization Header",
                    "Notion integration token as JSON header",
                )
                .placeholder(r#"{"Authorization": "Bearer ntn_..."}"#)
                .sensitive(),
            ],
        ),
        npx(
            "google-drive",
            "Google Drive",
            "Search and access Google Drive files.",
            "externaldrive.badge.icloud",
            Productivity,
            "@modelcontextprotocol/server-gdrive",
            vec![
                key("GDRIVE_CLIENT_ID", "Client ID", "Google OAuth client ID")
                    .placeholder("123456789.apps.googleusercontent.com"),
                key("GDRIVE_CLIENT_SECRET", "Client Secret", "Google OAuth client secret").sensitive(),
            ],
        ),
        npx(
            "brave-search",
            "Brave Search",
            "Web and local search powered by Brave.",
            "magnifyingglass",
            Data,
            "@modelcontextprotocol/server-brave-search",
            vec![key("BRAVE_API_KEY", "API Key", "Brave Search API key").sensitive()],
        ),
        npx(
            "fetch",
            "Fetch",
            "Fetch and convert web content to markdown.",
            "arrow.down.doc",
            Data,
            "@anthropic-ai/mcp-server-fetch",
            vec![],
        ),
        npx(
            "postgres",
            "PostgreSQL",
            "Query and inspect PostgreSQL databases.",
            "cylinder",
            Data,
            "@modelcontextprotocol/server-postgres",
            vec![
                key(
                    "POSTGRES_CONNECTION_STRING",
                    "Connection String",
                    "PostgreSQL connection URI",
                )
                .placeholder("postgresql://user:pass@localhost:5432/db")
                .sensitive(),
            ],
        ),
        npx(
            "sqlite",
            "SQLite",
            "Query and manage SQLite databases.",
            "tablecells",
            Data,
            "@modelcontextprotocol/server-sqlite",
            vec![
                key("SQLITE_DB_PATH", "Database Path", "Path to the SQLite database file")
                    .placeholder("/path/to/database.db"),
            ],
        ),
        npx(
            "filesystem",
            "Filesystem",
            "Secure read/write access to specified directories.",
            "folder.badge.gearshape",
            Filesystem,
            "@modelcontextprotocol/server-filesystem",
            vec![
                key(
                    "ALLOWED_DIRS",
                    "Allowed Directories",
                    "Comma-separated list of allowed directory paths",
                )
                .placeholder("/Users/me/projects,/tmp"),
            ],
        ),
        npx(
            "memory",
            "Memory",
            "Persistent memory using a knowledge graph.",
            "brain",
            Filesystem,
            "@modelcontextprotocol/server-memory",
            vec![],
        ),
        npx(
            "everart",
            "EverArt",
            "AI image generation via EverArt models.",
            "paintpalette",
            Data,
            "@modelcontextprotocol/server-everart",
            vec![key("EVERART_API_KEY", "API Key", "EverArt API key").sensitive()],
        ),
    ]
}

/// Case-insensitive substring match on name, description or category label; an empty query returns everything.
pub fn search(query: &str) -> Vec<McpCatalogEntry> {
    let q = query.trim().to_lowercase();
    entries()
        .into_iter()
        .filter(|e| {
            q.is_empty()
                || e.name.to_lowercase().contains(&q)
                || e.description.to_lowercase().contains(&q)
                || e.category.label().to_lowercase().contains(&q)
        })
        .collect()
}

/// Entries grouped by category in category order, skipping empty categories.
pub fn grouped() -> Vec<(McpCatalogCategory, Vec<McpCatalogEntry>)> {
    let all = entries();
    McpCatalogCategory::ALL
        .into_iter()
        .map(|c| (c, all.iter().filter(|e| e.category == c).cloned().collect::<Vec<_>>()))
        .filter(|(_, v)| !v.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn entry(id: &str) -> McpCatalogEntry {
        entries().into_iter().find(|e| e.id == id).unwrap()
    }

    // Ported from PoirotTests/Models/MCPServerCatalogTests.swift.
    #[test]
    fn catalog_has_entries() {
        assert_eq!(entries().len(), 15);
    }
    #[test]
    fn catalog_all_entries_have_unique_ids() {
        let ids: HashSet<_> = entries().into_iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), entries().len());
    }
    #[test]
    fn catalog_all_entries_have_required_fields() {
        assert!(entries().iter().all(|e| !e.id.is_empty()
            && !e.name.is_empty()
            && !e.description.is_empty()
            && !e.icon.is_empty()
            && !e.command.is_empty()));
    }
    #[test]
    fn search_empty_query_returns_all() {
        assert_eq!(search("").len(), entries().len());
    }
    #[test]
    fn search_by_name_finds_match() {
        assert!(search("github").iter().any(|e| e.id == "github"));
    }
    #[test]
    fn search_by_description_finds_match() {
        assert!(search("browser automation").iter().any(|e| e.id == "playwright"));
    }
    #[test]
    fn search_no_match_returns_empty() {
        assert!(search("xyznonexistent123").is_empty());
    }
    #[test]
    fn search_case_insensitive() {
        assert!(search("GITHUB").iter().any(|e| e.id == "github"));
    }
    #[test]
    fn grouped_covers_all_entries() {
        assert_eq!(grouped().iter().map(|(_, v)| v.len()).sum::<usize>(), entries().len());
    }
    #[test]
    fn grouped_no_category_is_empty() {
        assert!(grouped().iter().all(|(_, v)| !v.is_empty()));
    }
    #[test]
    fn categories_all_have_icons() {
        assert!(McpCatalogCategory::ALL.iter().all(|c| !c.icon().is_empty()));
    }
    #[test]
    fn env_keys_github_requires_token() {
        let keys = entry("github").env_keys;
        assert_eq!(keys.len(), 1);
        assert!(keys[0].is_required && keys[0].is_sensitive);
    }
    #[test]
    fn env_keys_playwright_requires_none() {
        assert!(entry("playwright").env_keys.is_empty());
    }
    #[test]
    fn env_keys_all_have_unique_ids_per_entry() {
        for e in entries() {
            let ids: HashSet<_> = e.env_keys.iter().map(|k| &k.id).collect();
            assert_eq!(ids.len(), e.env_keys.len(), "duplicate env key id in {}", e.id);
        }
    }

    // Ported from the catalog cases in PoirotTests/Services/SettingsWriterMCPTests.swift.
    #[test]
    fn catalog_entries_are_valid() {
        assert!(entries().iter().all(|e| !e.command.is_empty() && e.is_npx()));
    }
    #[test]
    fn catalog_search_by_category() {
        let results = search("Developer");
        assert!(!results.is_empty());
        assert!(results.iter().all(|e| e.category == McpCatalogCategory::Developer));
    }
    #[test]
    fn env_key_default_values() {
        let k = McpEnvKey::new("TEST_KEY", "Test", "A test key");
        assert!(k.is_required && !k.is_sensitive && k.placeholder.is_empty());
    }
    #[test]
    fn env_key_custom_values() {
        let k = McpEnvKey::new("SECRET", "Secret", "A secret")
            .placeholder("sk-...")
            .optional()
            .sensitive();
        assert!(!k.is_required && k.is_sensitive);
        assert_eq!(k.placeholder, "sk-...");
    }
}
