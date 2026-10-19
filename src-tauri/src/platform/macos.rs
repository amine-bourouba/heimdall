//! macOS launch execution: no OS-specific preparation needed before a spawn (M3-52).

pub fn prepare(_command: &mut std::process::Command) {}
