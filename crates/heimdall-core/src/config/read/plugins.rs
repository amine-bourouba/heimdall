//! Installed plugins (ClaudeConfigLoader.loadPlugins): `plugins/installed_plugins.json`, with
//! each plugin's enabled flag from the global `settings.json`.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::{settings, sort_by_name};
use crate::error::Loaded;
use crate::models::config::plugin::ClaudePlugin;
use crate::paths::ClaudeHome;

#[derive(Deserialize, Default)]
#[serde(default)]
struct InstalledPlugins {
    plugins: BTreeMap<String, Vec<PluginVersion>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct PluginVersion {
    version: Option<String>,
    installed_at: Option<String>,
}

/// One entry per installed version of each `<name>@<author>` key, sorted by name. `skipped` is 1
/// when the file exists but does not parse.
pub fn load(home: &ClaudeHome) -> Loaded<Vec<ClaudePlugin>> {
    let Ok(bytes) = std::fs::read(home.installed_plugins_file()) else { return Loaded::new(Vec::new(), 0) };
    let Ok(installed) = serde_json::from_slice::<InstalledPlugins>(&bytes) else { return Loaded::new(Vec::new(), 1) };
    let enabled = enabled_plugins(home);
    let mut items: Vec<ClaudePlugin> = installed
        .plugins
        .into_iter()
        .flat_map(|(key, versions)| {
            let (name, author) = key.split_once('@').map_or((key.clone(), String::new()), |(n, a)| (n.to_string(), a.to_string()));
            let is_enabled = enabled.get(&key) == Some(&true);
            versions.into_iter().map(move |v| ClaudePlugin {
                id: key.clone(),
                name: name.clone(),
                author: author.clone(),
                version: v.version.unwrap_or_default(),
                installed_at: v.installed_at.unwrap_or_default(),
                is_enabled,
            })
        })
        .collect();
    sort_by_name(&mut items, |p| &p.name);
    Loaded::new(items, 0)
}

/// `enabledPlugins` of the global `settings.json`.
pub fn enabled_plugins(home: &ClaudeHome) -> BTreeMap<String, bool> {
    settings::load(&home.settings_file()).ok().flatten().and_then(|s| s.enabled_plugins).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::synthetic;

    #[test]
    fn plugins_carry_their_enabled_flag() {
        let home = ClaudeHome::new(synthetic().join("claude-home"), synthetic().join("claude.json"));
        let items = load(&home).value;
        let got: Vec<_> = items.iter().map(|p| (p.name.as_str(), p.author.as_str(), p.version.as_str(), p.is_enabled)).collect();
        assert_eq!(got, [("code-review-bundle", "acme", "1.4.0", true), ("legacy-tools", "acme", "0.9.2", false)]);
    }
}
