# Upstream sync procedure

Task M0-16. Heimdall ports Poirot as of upstream commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86` (https://github.com/a7t-ai/poirot). This procedure finds what upstream has changed since, so genuinely new capabilities can enter the plan.

## Cadence

Run it **monthly**, and also before each milestone gate review (M1-59, M2-30, M3-20, M4-14).

## Commands

The Swift reference checkout (`$POIROT`) is read-only, so do not fetch into it. Use a throwaway clone instead:

```bash
PIN=a29dd6ee0e0133ef26bd067c164c66b160e13d86
TMP=$(mktemp -d)
git clone -q --filter=blob:none --no-checkout https://github.com/a7t-ai/poirot.git "$TMP/poirot"
git -C "$TMP/poirot" rev-parse origin/HEAD                      # upstream head
git -C "$TMP/poirot" log --oneline $PIN..origin/HEAD -- Poirot/Sources README.md SHOWCASE.md
git -C "$TMP/poirot" diff --stat $PIN..origin/HEAD -- Poirot/Sources
```

A checkout that is allowed to change can use the equivalent `git -C $POIROT fetch origin` followed by `git -C $POIROT log --oneline a29dd6e..origin/main -- Poirot/Sources README.md SHOWCASE.md`.

## What to do with the results

For each upstream commit found:

1. **New capability** (a screen, action, data source, preference or shortcut): add a new row to `docs/parity-matrix.md` in the matching section, with scope `MVP` or `Later`, and a new task in `docs/tasks/`. Never reopen a finished milestone; new rows land in the current or a later milestone.
2. **Change to a file format or path Poirot reads**: update `docs/spec/claude-home-inventory.md`, add or adjust fixtures, and open a task for the affected Rust loader.
3. **Bug fix or refactor with no behaviour change**: record it in the log below; no plan change.
4. **Removed capability** (as `68b7b0c` removed Usage Limits): mark the parity row out of scope with the commit as the reason.

Moving the pinned SHA is a deliberate change: architect-reviewer approves it, `docs/port-plan.md`, ADR 0001 and the golden-oracle workflow are updated together, and the oracle is re-run (port-plan §13).

## Log

| Date | Upstream head | Commits since pin (all paths) | Commits touching `Poirot/Sources`, `README.md`, `SHOWCASE.md` | Proposed matrix rows |
|---|---|---|---|---|
| 2026-09-25 | `a29dd6ee0e0133ef26bd067c164c66b160e13d86` | 0 | 0 | none |
