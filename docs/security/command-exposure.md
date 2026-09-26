# Command exposure across windows (Tauri 2, ADR 0010)

Status: recorded by task M1-49. Cites the pinned Tauri version's own source
(`tauri = "2.11.6"`, `Cargo.lock` pins `2.11.6` exactly) and its docs at that
version. The manual devtools check M1-48's acceptance criterion asks for
(calling `update_prefs` from the tray webview) is recorded in that task's
progress-log entry, once `capabilities/main.json`/`tray.json` exist to make
the check meaningful.

## Docs cited

- [Capabilities](https://v2.tauri.app/security/capabilities/) — Tauri v2 docs, Security section.
  Confirms capabilities are per-window/per-webview (a `windows` array on each
  capability file) and that "windows and webviews which are part of more than
  one capability effectively merge the security boundaries and permissions of
  all involved capabilities" — the reason the tray window must only ever be
  listed in `tray.json`, never also matched by `main.json`.
- [Permissions](https://v2.tauri.app/security/permissions/) — the
  `${permission-name}` (app commands) vs `${plugin-name}:${permission-name}`
  (plugin commands) naming rule capability files use to grant a command.

Neither docs page states the exact runtime behaviour for a denied call (error
type, message, or *when* the check happens relative to the command handler
running) — verified instead by reading `tauri` 2.11.6's own source, cited
below with exact file and line numbers.

## What actually happens on a denied call (traced in `tauri` 2.11.6's source)

`src/webview/mod.rs:1794-1850` (the IPC message handler every `invoke()` call
goes through) resolves an ACL grant for `(command, window, webview, origin)`
*before* dispatching to the command's Rust function at all:

```rust
let (resolved_acl, has_app_acl_manifest) = {
    let runtime_authority = manager.runtime_authority.lock().unwrap();
    let acl = runtime_authority.resolve_access(&request.cmd, ..., &acl_origin);
    (acl, runtime_authority.has_app_manifest())
};
...
if (plugin_command.is_some() || has_app_acl_manifest || !is_local) && ... && invoke.acl.is_none() {
    invoke.resolver.reject(format!("Command {} not allowed by ACL", request.cmd)); // release builds
    return; // the command's own handler is never called
}
```

- **The check is a hard gate, not a fallback.** `invoke.acl.is_none()` (no
  capability anywhere granted this exact command to this exact window/webview)
  rejects the JS-side promise and `return`s immediately — the app's command
  function never runs. The webview only ever sees a rejected promise, never a
  panic or a partial response.
- **Release builds** reject with the plain string `"Command <name> not
  allowed by ACL"`. **Debug builds** (`npx tauri dev`, what a manual devtools
  check in M1-48 uses) call `resolve_access_message` instead, which — per its
  own doctest at `src/ipc/authority.rs:1140-1191` — names the window, webview,
  URL context, what *would* have allowed it, and which capability/permission
  was checked, e.g.:

  ```text
  myplugin.my-command-window not allowed on window "other-window", webview "any-webview", URL: local

  allowed on: [windows: "main-*", URL: local]

  referenced by: capability: maincap, permission: allow-command
  ```

  So M1-48's manual check does not just need to see a rejection — the debug
  message itself should name the *tray* window/webview and confirm no
  capability granting `update_prefs` matched it.

## Why M1-49 (this task) is not just bookkeeping

The gate above only fires unconditionally for **plugin** commands and for any
**non-local** origin. For an app-defined command (`get_app_info`,
`update_prefs`, …) called from a **local** origin, the deny branch is only
reached when `has_app_acl_manifest` is true — and that flag is exactly
`RuntimeAuthority::has_app_manifest()`, which reflects whether the app called
`AppManifest::commands(...)` in `build.rs` at all (`src/webview/mod.rs:1794-1802`).

Concretely: **before this task, `capabilities/main.json`/`tray.json` (M1-48)
would have had no effect on Heimdall's own commands at all.** Every app
command would have been reachable from every window regardless of what any
capability file said, because `has_app_acl_manifest` was false (`build.rs` was
still the M1-03 placeholder, `tauri_build::build()` with no `AppManifest`).
`build.rs` now calls `AppManifest::new().commands(COMMANDS)` with the exact 65
commands `docs/ipc-contract.md` §6 lists, generating an
`allow-<command>`/`deny-<command>` permission pair per command for
`capabilities/*.json` to reference by name — this is the switch that turns
the ACL gate on for Heimdall's own commands in the first place, not merely a
list capability files could already have relied on.

## Verified by

- `commands::tests::build_rs_lists_exactly_the_contract_commands` (in
  `src-tauri/src/commands/mod.rs`) parses `build.rs`'s own `COMMANDS` array
  out of its source text and compares it, sorted, against
  `CONTRACT_COMMAND_NAMES` — the same 65-name list M1-37 froze from the
  contract table. `cargo check -p heimdall-desktop`/`cargo clippy
  --workspace --all-targets -- -D warnings` both exit 0; the test itself
  could not be executed locally, per the `STATUS_ENTRYPOINT_NOT_FOUND`
  constraint documented since M1-37 (every test in the `heimdall-desktop`
  crate hits it, regardless of what the individual test touches).
- The source-reading above (not a doc citation) is this task's own
  "experiment showing a command absent from a window's capability is
  refused": traced exactly what `webview/mod.rs` does end to end, rather than
  asserting it from the docs alone, since the docs page for capabilities
  does not describe the runtime denial path at all.
