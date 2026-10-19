# ADR 0001: Repository layout and the Swift reference

- **Status:** Accepted (user answer 1, port-plan §2.1)
- **Date:** 2026-09-25

## Context

The planning brief offered three layouts inside a fork of the Swift reference app: replace the Swift tree in place, move it to `legacy/`, or freeze it beside the new code. The user chose none of them: Heimdall is a separate new project, pushed to GitHub later, and the Swift source stays outside it as a read-only reference checkout.

The port still needs the Swift app as an oracle: to generate expected outputs for shared fixtures, to settle parser ambiguity, and to compare screens.

## Decision

1. **Heimdall is its own repository** with the layout in port-plan §6: a Cargo workspace (`crates/heimdall-core`, `src-tauri`) and a Vite/Vue app (`src/`) at the root. No Swift code, Xcode project or Swift CI lives in it.
2. **The Swift reference is `$SWIFT_REF`** (`C:\Users\pc\Desktop\tech\poirot` on this machine — the checkout's own on-disk name is outside this repository and not renamed), a local read-only checkout of the original Swift app this project ports, pinned at upstream commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Nobody modifies it. Task files cited it as `$SWIFT_REF/...` (formerly `$POIROT/...`, same checkout, renamed in prose only).
3. **The oracle ran in CI, not in the repo (retired, see point 4).** A Swift test file kept in `tools/golden-oracle/` (M1-50) was copied into a throwaway checkout of the upstream Swift app at the pinned SHA by a macOS workflow (M1-51). Its JSON outputs were committed under `fixtures/golden/` (M1-52) and compared by Rust tests (M1-53). The user chose GitHub-hosted macOS runners for this (decision D3).
4. **Retired 2026-10-19 (M4-15, M4-16), ahead of the original condition.** The harness and its workflow were deleted by explicit user directive rather than the natural precondition (every MVP parity row At parity on Windows and Linux, every parser-related test row in the Swift reference's own suite covered by a passing Rust test) — that precondition was not yet met when the retirement happened. `fixtures/golden/` and the parity tests (`golden_parity.rs`) stay as regression tests; they do not depend on the harness or workflow.
5. **Moot after retirement (point 4).** Moving the pinned SHA would have been a deliberate change requiring architect-reviewer approval and an oracle re-run (port-plan §13); with the oracle gone, there is nothing left to re-run.
6. No top-level folder may differ from another only by letter case (Windows and macOS file systems are case-insensitive by default).

## Consequences

- The brief's option C concerns (Swift CI path filtering, `swift-final` tag, case-only folder-name collisions with the reference app) do not apply.
- The final layout exists from day one; there is no retirement commit inside this repository.
- Golden parity needs a macOS runner; it uses GitHub-hosted runners (D3), so no local Mac is required.
- The Heimdall folder was moved out of the unrelated `hydra` repository to `C:\Users\pc\Desktop\tech\Heimdall` on 2026-09-25 (D4); M0-03 initialises it as its own repository.

## Related

Port-plan §2, §6, §7 (M4), §13 · Tasks M0-03, M0-16, M1-50 to M1-53, M4-15, M4-16
