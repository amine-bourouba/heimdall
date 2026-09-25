# ADR 0011: App identifier and release repository

- **Status:** Accepted (identifier and Linux name chosen by the user on 2026-09-25, answers D1 and D2; recorded by task M0-02)
- **Date:** 2026-09-25

## Context

Poirot's `project.yml:41` sets `PRODUCT_BUNDLE_IDENTIFIER: fyi.poirot-54`, a leftover per-worktree value, not a real identifier. Its update checker points at `a7t-ai/poirot` (`Services/UpdateChecker.swift:5`), and its About view links to the upstream maintainer's repository, Sponsors page and social profile. The OS keys each app's preferences folder by identifier, so changing it after release orphans user settings (ADR 0007).

The user named the product **Heimdall**, chose **https://github.com/amine-bourouba/heimdall** as the release repository, and asked for a Linux package-name collision check.

## Decision

### Settled

- **Product name:** Heimdall (window titles, installers, About).
- **Release repository:** `github.com/amine-bourouba/heimdall`. The updater endpoint is `https://github.com/amine-bourouba/heimdall/releases/latest/download/latest.json`. About and Help link only to this repository; the upstream maintainer's Sponsors and social links are not carried over.
- **Distribution:** GitHub Releases only (user answer 7).
- **Signing:** Windows unsigned; macOS ad-hoc signed and not notarised; no upstream Apple secrets are reused (user answer 7). A Tauri updater key is generated and stored as CI secrets (M4-02).
- **Architectures:** macOS `aarch64-apple-darwin` only (user answer 8); Windows and Linux `x86_64` only at launch, ARM64 later (user answer 9).
- **Copyright holder for the Heimdall line in LICENSE:** the repository owner, `amine-bourouba`, unless the user supplies a different name.

### Identity

| Item | Value | Source |
|---|---|---|
| App identifier | `io.github.amine-bourouba.heimdall` | D1 |
| Linux package name | `heimdall-desktop` (deb and rpm) | D2 |
| Binary name | `heimdall-desktop` on every OS, so one Cargo binary name serves all platforms; installers, the macOS bundle and window titles still say "Heimdall" | D2 |

The identifier uses only letters, digits, `.` and `-`, and does not end in `.app`, so Tauri, macOS, Windows and Linux all accept it.

**Collision evidence (checked 2026-09-25).** `heimdall` is taken; `heimdall-desktop` is not. Each search below lists every official package whose name contains "heimdall", and none is `heimdall-desktop`:

- Debian and Ubuntu: package `heimdall-flash` installs `/usr/bin/heimdall` (https://packages.debian.org/search?keywords=heimdall, file list for bookworm/amd64).
- Arch Linux: package `heimdall` in `extra` (https://archlinux.org/packages/?q=heimdall).
- Fedora: package `heimdall` (https://packages.fedoraproject.org/search?query=heimdall).

The only matches are the Samsung Galaxy firmware flashing tool and its Qt front end (`heimdall-flash-frontend`, binary `heimdall-frontend`), plus Fedora's unrelated `heimdal-server`. A package or binary named `heimdall` would conflict at install time or on `PATH`; `heimdall-desktop` does not.

### Development identifiers

Development builds use `io.github.amine-bourouba.heimdall.dev`; worktree N uses `io.github.amine-bourouba.heimdall.dev<N>` (port-plan §5.8).

## Consequences

- M1-01 (crate and binary names), M1-03 (identifier) and M4-01 (Linux package name) use these values.
- Once the first tagged release ships, the identifier is effectively permanent.

## Related

Port-plan §2.1, §2.2 (D1, D2) · ADR 0007 · Tasks M0-02, M0-04, M1-01, M1-03, M4-01, M4-03, M3-17
