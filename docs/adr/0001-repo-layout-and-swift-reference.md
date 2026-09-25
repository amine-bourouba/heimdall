# ADR 0001: Repository layout and the Swift reference

- **Status:** Accepted (user answer 1, port-plan §2.1)
- **Date:** 2026-09-25

## Context

The planning brief offered three layouts inside a fork of Poirot: replace the Swift tree in place, move it to `legacy/`, or freeze it beside the new code. The user chose none of them: Heimdall is a separate new project, pushed to GitHub later, and the Swift source stays outside it as a read-only reference at `C:\Users\pc\Desktop\tech\poirot`.

The port still needs the Swift app as an oracle: to generate expected outputs for shared fixtures, to settle parser ambiguity, and to compare screens.

## Decision

1. **Heimdall is its own repository** with the layout in port-plan §6: a Cargo workspace (`crates/heimdall-core`, `src-tauri`) and a Vite/Vue app (`src/`) at the root. No Swift code, Xcode project or Swift CI lives in it.
2. **The Swift reference is `$POIROT`**, pinned at upstream commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86` (https://github.com/a7t-ai/poirot). Nobody modifies it. Task files cite it as `$POIROT/...`.
3. **The oracle runs in CI, not in the repo.** A Swift test file kept in `tools/golden-oracle/` (M1-50) is copied into a throwaway checkout of upstream Poirot at the pinned SHA by a macOS workflow (M1-51). Its JSON outputs are committed under `fixtures/golden/` (M1-52) and compared by Rust tests (M1-53). The user chose GitHub-hosted macOS runners for this (decision D3).
4. **Retirement is a condition, not a date.** The harness and its workflow are deleted (M4-15, M4-16) only when every MVP parity row is At parity on Windows and Linux and every parser-related PoirotTests row is covered by a passing Rust test. `fixtures/golden/` and the parity tests stay as regression tests.
5. **Moving the pinned SHA** is a deliberate change: architect-reviewer approves it, and the oracle is re-run (port-plan §13).
6. No top-level folder may differ from another only by letter case (Windows and macOS file systems are case-insensitive by default).

## Consequences

- The brief's option C concerns (Swift CI path filtering, `swift-final` tag, `Poirot/` versus `poirot/` collisions) do not apply.
- The final layout exists from day one; there is no retirement commit inside this repository.
- Golden parity needs a macOS runner; it uses GitHub-hosted runners (D3), so no local Mac is required.
- The Heimdall folder was moved out of the unrelated `hydra` repository to `C:\Users\pc\Desktop\tech\Heimdall` on 2026-09-25 (D4); M0-03 initialises it as its own repository.

## Related

Port-plan §2, §6, §7 (M4), §13 · Tasks M0-03, M0-16, M1-50 to M1-53, M4-15, M4-16
