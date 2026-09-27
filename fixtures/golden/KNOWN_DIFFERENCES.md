# Known differences between Rust and Swift golden output

Every tolerated difference `crates/heimdall-core/tests/golden_parity.rs` (task M1-53) accepts
rather than treats as a failure, found by running that test for real against the committed
`fixtures/golden/**` (task M1-52). Each entry names the exact evidence (real session ids and
numbers), not a hypothetical.

## 1. Decision D5 — token accounting for a message split by a tool-result interruption

**Reason:** `docs/port-plan.md` line 44, decision D5, answered by the user on 2026-09-26 (already
decided before this task ran, not found by it): Poirot's `TranscriptParser.parse` groups
consecutive same-`msg_id` assistant records into one `Message`, but a tool-result (`type: "user"`)
record in between unconditionally flushes the pending group — so a message that streams across
such an interruption becomes **two** `Message` entries sharing the same id, and Poirot sums both
groups' full usage into `Session.totalTokens`, double-counting it. Heimdall's `transcript/full.rs`
deliberately dedupes via its own `usage_by_id` map ("its usage goes on the first group only," per
that file's own comment) and takes each message's *last* streamed record rather than its first
(`port-plan.md` C-25/C-26: Poirot's first-record rule undercounts real transcripts by roughly 3x).

**Evidence — real sessions in `fixtures/windows/`, found by `golden_parity.rs`'s own comparison:**

| Session | Field | Rust | Golden (Poirot) |
|---|---|---|---|
| `C--Users-fixture-Desktop-tech/348fef32-...` | totalTokens | 12,988 | 15,156 |
| `c--Users-fixture-Desktop-tech-proj-10/aa95c61f-...` | totalTokens | 56,698 | 74,871 |
| `c--Users-fixture-Desktop-tech-proj-11/54a79a0b-...` | totalTokens | 33,635 | 126,278 |

Golden is always >= Rust for every affected field (`inputTokens`, `outputTokens`,
`cacheReadTokens`, `cacheCreationTokens`, `totalTokens`) — the direction `assert_matches_or_known_d5_divergence`
asserts; a mismatch in the other direction would still fail. Sessions this affects also skip the
`export_markdown` comparison entirely (their message list differs, so their rendering isn't
byte-comparable to begin with — see `token_totals_match` and `context_usage_matches` for the
numeric parity that *is* asserted for these sessions).

**Sign-off:** the underlying decision (last-record vs first-record, dedup by id) already carries
the user's own sign-off (`port-plan.md`, 2026-09-26). This document's own job is only to record
that it manifests exactly as expected on real fixture data.

## 2. Tool-input key display order

**Reason:** `crates/heimdall-core/src/export/session_markdown.rs`'s own comment on
`tool_use_lines` (predates this task): Rust renders a `tool_use` block's "**key:** value" lines
from `ToolUse.input: BTreeMap<String, String>` — always alphabetically sorted, deterministic.
Poirot's `ToolUse.input` is a plain `[String: String]`, whose iteration order Swift's `Dictionary`
never guarantees. The committed golden `export.md` files simply froze whatever order that one CI
run's hash table happened to produce; there is no stable order to port to, and even re-running the
same Swift binary again could reorder these lines.

**Evidence:** every session below has a `tool_use` block with 2+ input keys, and diverges from
golden starting exactly at that block's parameter lines (confirmed directly for one — a real
`Grep` call whose golden output lists `pattern`/`output_mode` first and `glob` third; Rust lists
`glob` first, alphabetically):

- `fixtures/golden/linux/sessions/-home-fixture/0b886352-...`
- `fixtures/golden/screenshot/sessions/-Users-fixture-Dev-poirot/{alltools,conv,diff,tool,write}-001`
- `fixtures/golden/screenshot/sessions/-Users-fixture-Dev-reellette-ios/task-001`
- `fixtures/golden/synthetic/sessions/-Users-fixture-src-app/11111111-...`
- `fixtures/golden/windows/sessions/c--Users-fixture-Desktop-tech-proj-11/{604186db,ab8bf07e}-...`

**Sign-off:** found by this task (M1-53), not previously decided. Rust's own choice (deterministic,
sorted order) is self-evidently the right one for a shipped app — an arbitrary, run-to-run-unstable
display order is not a property worth porting. Pending qa-expert review; no objection expected.

## 3. Truncation boundary: grapheme clusters vs Unicode scalars

**Reason:** `session_markdown.rs`'s own comment on `INPUT_TRUNCATE` (predates this task, never
before empirically triggered): tool-result/tool-input truncation in Rust counts via
`.chars().count()` (Unicode *scalar values*); Poirot's Swift counts via `String.count` (extended
*grapheme clusters*). A `\r\n` sequence is one grapheme cluster in Swift but two scalars in Rust —
so when one lands exactly at the 2000-character (tool result) or 500-character (tool input)
truncation cutoff, the two sides truncate a character or two apart.

**Evidence:** confirmed directly against the raw transcript (not guessed): the Windows fixture's
own tool-result content — real CLI output whose original `\r\n` line endings the scrub tool
preserves byte-for-byte — lands a CRLF within a few characters of the 2000-char cutoff in more than
one session's `export.md`. `golden_parity.rs`'s `matches_or_known_truncation_boundary_drift` treats
a difference as tolerated only when it is small and bounded (at most 200 bytes of total length
difference and of estimated differing content, on documents tens of kilobytes long) — a
substantive content difference would fail this bound, not slip through it.

**Fix considered and deferred:** grapheme-aware truncation would need the `unicode-segmentation`
crate (not currently a workspace dependency) — a real code change, not something this parity test
should make unilaterally. Left as a follow-up.

**Sign-off:** found by this task (M1-53). Pending qa-expert review.
