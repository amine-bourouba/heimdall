//! Windows launch execution (M3-52's own AC1: `CREATE_NO_WINDOW`).
#![allow(dead_code)] // no caller until the launch commands (M3-53) land

/// Suppresses the console window Windows would otherwise flash open for a console-subsystem
/// program (`cmd.exe`, `powershell.exe`) -- harmless to apply unconditionally to every spawn,
/// including GUI programs (`wt.exe`, editors), which never had a console to suppress in the first
/// place. `0x0800_0000` is `CREATE_NO_WINDOW` (`windows::Win32::System::Threading::CREATE_NO_WINDOW`,
/// confirmed against that crate's own source rather than assumed) -- the raw value is used
/// directly rather than adding a dependency on the `windows` crate to this binary crate just for
/// one well-known, stable constant, matching `heimdall-core`'s own `atomic.rs` precedent of
/// documenting a raw OS-level value in a comment instead of importing a symbol for it.
pub fn prepare(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}
