# fixtures/

Every fixture set Heimdall's tests read from disk. Nothing under `fixtures/` is
generated at test time except `fixtures/dto/` (regenerated from `heimdall-core`
by an explicit script, see below); everything else is committed as captured or
hand-built bytes.

## Fixture sets

| Set | What it is | Owning task(s) |
|---|---|---|
| `fixtures/synthetic/` | Hand-built, deterministic fixtures: one folder per `PoirotTests/Services` (and JSON-decoding `PoirotTests/Models`) test case under `cases/`, plus one integrated synthetic `~/.claude` tree (`claude-home/`, `claude.json`, `project-repo/`) covering every format in `docs/spec/claude-home-inventory.md`, plus `corrupt/` for intentionally unparsable files. Home directory: `/Users/fixture`. | M0-11 (create); M0-14, M0-15 depend on it existing first |
| `fixtures/screenshot/` | Deterministic fixture tree converted from Poirot's `ScreenshotData.swift`, used by Playwright visual tests. Home directory: `/Users/fixture`. | M0-12 |
| `fixtures/windows/` | A scrubbed copy of a real Claude Code home captured on Windows 11, run through the scrub tool. Home directory: `C:\Users\fixture`. See `fixtures/windows/MANIFEST.md` for capture details, format facts and CRLF handling. | M0-13 (scrub tool), M0-14 (capture) |
| `fixtures/linux/` | A scrubbed copy of a real Claude Code home captured on Linux, run through the same scrub tool. Home directory: `/home/fixture`. | M0-13 (scrub tool), M0-15 (capture) |
| `fixtures/golden/` | JSON outputs from the Swift reference app (the "golden oracle"), committed with their provenance, used to prove Rust output equals Swift output. Also `fixtures/golden/KNOWN_DIFFERENCES.md` for any accepted, justified difference. | M1-50 to M1-52 (produce), M1-53 (consume), M1-59 (gate) |
| `fixtures/dto/` | Generated, not hand-written: `cargo run -p heimdall-core --example export_dto_fixtures` regenerates it byte-identically from `heimdall-core`'s DTOs. Vitest and the IPC mock layer (`src/ipc/testing.ts`) read it. A drift check in CI (M2-29) fails if the committed copy and a fresh regeneration differ. | M2-01 (create + regenerate script), M2-03 / M2-23 (consume) |
| `fixtures/security/` | Adversarial fixtures: `secrets-claude.json` (a `~/.claude.json` stand-in whose secrets must never reach a DTO or a log line), `xss-payloads.json` (a corpus of XSS/injection strings), `malicious-claude-home/` (a `~/.claude` tree built entirely from `xss-payloads.json` content, for the malicious-transcript E2E test). | M1-55, M2-25 (create), M2-26, M3-35 (consume) |

## Rules

- **Byte-exact.** `.gitattributes` marks `fixtures/** -text`: git never touches
  line endings inside this tree. A transcript captured with CRLF stays CRLF;
  never re-save a fixture file through an editor that normalises endings.
- **No real personal data.** Every captured fixture (`windows/`, `linux/`) is
  scrubbed first (task M0-13): usernames, emails, tokens, real file contents
  and real project names are replaced with pseudonyms before the capture is
  committed. Hand-built fixtures (`synthetic/`, `screenshot/`, `security/`)
  never contain real data to begin with; secrets use sentinel values such as
  `SENTINEL-SECRET-1`.
- **Home paths are fixed per platform, and only that one path appears:**
  - `/Users/fixture` — `fixtures/synthetic/`, `fixtures/screenshot/`
  - `C:\Users\fixture` — `fixtures/windows/`
  - `/home/fixture` — `fixtures/linux/`

  No fixture may contain any other absolute home-directory path. Project path
  segments inside a captured fixture are replaced with stable pseudonyms
  (`proj-1`, `proj-2`, ...) by the scrub tool; hand-built fixtures use plain,
  obviously-fake project names (for example `/Users/fixture/src/app`).
- **One owner per set.** The task column above is the only place allowed to
  create or regenerate a fixture set's committed files. Consuming tasks read
  fixtures; they never edit them by hand.
