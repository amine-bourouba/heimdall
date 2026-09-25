//! Config scope (Models/Config/ConfigScope.swift): `~/.claude/` or `<project>/.claude/`.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConfigScope {
    Global,
    Project,
}
