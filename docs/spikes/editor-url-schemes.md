# Spike: editor URL schemes per OS (M3-54)

ADR 0003's Launchers table names `vscode://file/<abs>:<line>`, `cursor://file/...`,
`zed://file/...` for Windows, "URL schemes, else CLI found by PATH search" for Linux, and the
same plus `xed` for macOS — each "where M3-54 confirms them." This spike actually triggers those
schemes rather than trusting the pattern by analogy, on whatever this task could reach: this dev
machine's own Windows 11 install, with VS Code (the only one of the three editors actually
installed here).

## Method

Real files were created under a temp directory, including one path with both a space and
non-ASCII characters (`dir with spaces\café résumé\test_file.txt`), matching this task's own "including
paths with spaces and non-ASCII characters" wording. Each scheme/flag was invoked directly against
the installed editor's own executable (`Code.exe`), and the result was confirmed visually with a
real screenshot (not inferred from exit codes alone) — each screenshot shows the editor's own
status bar reporting the exact cursor line reached.

**A real methodological pitfall, worth recording for whoever re-runs this on a different
machine**: this spike was run from inside a shell whose own environment inherits
`ELECTRON_RUN_AS_NODE=1` (set by VS Code's own extension-host process, which the testing
environment happens to run inside). Any child `Code.exe` invocation that inherits this variable
silently runs as a plain Node.js CLI instead of launching the real editor — no error dialog, just
Node's own `--help`/`--version` output and a `bad option: --open-url` message for an unrecognised
flag. The first attempt at every test below hit this silently and looked like "nothing happened";
clearing `$env:ELECTRON_RUN_AS_NODE` before each invocation fixed it. Heimdall's own shipped Rust
process never sets or inherits this variable, so this is purely a spike-testing artifact, not
anything Heimdall itself needs to guard against — but it is exactly the kind of false negative
that would otherwise produce a wrong "scheme doesn't work" conclusion.

## Results

| Editor | OS | Scheme/command tried | Result |
|---|---|---|---|
| VS Code | Windows | `vscode://file/<abs>:<line>` (via `Code.exe --open-url -- <uri>`, matching the registered `HKCU:\Software\Classes\vscode\shell\open\command`) | **Works.** Opens the file with the cursor on the exact requested line (confirmed via the editor's own "Ln N, Col 1" status-bar readout) for both a plain path and the space/non-ASCII path. **A real, previously undocumented UX fact**: the first time a given path is opened this way in a session, VS Code shows a native confirmation dialog — *"An external application wants to open '\<path\>' in Code... If you did not initiate this request... you should press 'No'"* — with a "Allow opening local paths without asking" checkbox. Every `open_external_url`-style launch from Heimdall will trigger this unless the user checks that box once; this is VS Code's own behaviour, not something Heimdall's launcher code can skip. |
| VS Code | Windows | `vscode://file/<abs-with-spaces-and-non-ASCII>:<line>` (raw, unencoded space and non-ASCII characters embedded directly in the URI, not percent-encoded) | **Works.** The confirmation dialog displayed the full path correctly (`café résumé` rendered correctly, not mangled), and the file opened with the cursor on the exact requested line. Raw embedding (no percent-encoding needed) is sufficient for this registered handler on Windows. |
| VS Code | Windows | CLI fallback: `Code.exe -g <abs>:<line>` (the real `.exe`, never the `.cmd` shim — ADR 0010 point 7) | **Works, and with no confirmation dialog at all** (a direct CLI argument isn't treated as an "external request" the way the URL-protocol path is) — cursor lands on the exact requested line. This is the fallback argv Heimdall should use if the `vscode://` protocol is ever unregistered: `["-g", "<abs-path>:<line>"]` against the real VS Code executable path, resolved via PATH search, never `code.cmd`. |
| Cursor | Windows | `cursor://file/...` | **Not tested — Cursor is not installed on this machine.** ADR 0003's own pattern (`cursor://file/...`) is Cursor's own documented scheme (it is a VS Code fork and inherits the same `--open-url`/URL-handler machinery), so the VS Code result above is suggestive but not confirmed for Cursor specifically. |
| Zed | Windows | `zed://file/...` | **Not tested — Zed is not installed on this machine**, and at the time of this spike Zed's own Windows build is still early/preview-quality upstream — worth re-verifying availability before relying on it. |
| Any | Linux | URL schemes, else CLI | **Not tested — no Linux machine available in this environment.** |
| Any | macOS | URL schemes, else CLI; Xcode via `xed` | **Not tested — no macOS machine available in this environment.** |

## What this does and doesn't prove

VS Code's own `vscode://file/<abs>:<line>` scheme is real and works exactly as ADR 0003 assumes,
including the harder space/non-ASCII path case this task's own AC calls out by name — that part of
the launcher design is now empirically confirmed, not just asserted by analogy with Cursor/Zed's
similar-looking schemes. The CLI fallback (`-g <path>:<line>`, never a `.cmd` shim) also works and
is the right fallback argv to implement for Windows if a future task needs one (ADR 0003's own
Launchers table doesn't currently list a Windows fallback at all — this spike's own finding is
that one exists and works cleanly, worth a future ADR update if a launcher task picks it up).

Cursor, Zed, and every behaviour on Linux and macOS remain unconfirmed — this spike proves the
*pattern* is sound for one real case, not that all nine editor×OS combinations in ADR 0003's table
are correct. The confirmation-dialog finding is new information ADR 0003 doesn't currently mention
at all and is worth folding into whichever task actually implements the launcher (M3-52/M3-53),
since it changes the expected user experience (a native OS/editor dialog interrupts the flow on
first use per path, not a silent background open).

## Follow-ups

- Re-run this spike (or extend it) on a real Linux machine (Cursor/VS Code's own scheme plus the
  documented CLI-search fallback) and a real macOS machine (including `xed` for Xcode) before
  M3-52/M3-53 ship those code paths — this spike's own Linux/macOS rows are open, not failing.
- Install Cursor and/or Zed on a future test pass to confirm their own `cursor://`/`zed://`
  handlers behave the same way VS Code's does, rather than assuming it from the shared fork
  lineage alone.
- M3-52/M3-53 should account for the confirmation-dialog UX this spike found (not in ADR 0003
  today) when documenting or testing the launch flow, and should implement the Windows CLI
  fallback (`-g <path>:<line>`, real `.exe` only) this spike confirmed works, for the case where
  the URL protocol handler isn't registered at all.
