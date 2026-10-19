//! Linux launch execution: no OS-specific preparation needed before a spawn (M3-52). The reveal
//! half of this task's own Linux row (degrading to the parent folder) lives in the opener plugin
//! itself, not here -- see `mod.rs`'s own `reveal` doc comment.

pub fn prepare(_command: &mut std::process::Command) {}
