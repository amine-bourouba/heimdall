# Session loading is slow when there are 100+ sessions per project. Can we optimize it?

**Project:** poirot
**Model:** claude-opus-4-6
**Date:** September 20, 2026 at 7:00 AM
**Turns:** 1
**Tokens:** 9.0k

---

## Turn 1 — User
*07:00:00*

Session loading is slow when there are 100+ sessions per project. Can we optimize it?


---

## Turn 1 — Assistant
*07:00:00*

Looking at the current `SessionLoader` and `TranscriptParser`, I can see the bottleneck clearly:

## Performance Analysis

Every session file (~2MB average) is fully parsed on launch. For 100+ sessions, that's **200+ MB of JSON** decoded before anything renders.

## Recommended Approach: Two-Phase Loading

### Phase 1 — Fast Metadata Scan
Read only the first 5 lines of each `.jsonl` for the sidebar summary (title, timestamp, model, turn count). This gives us **~50x speedup** for initial load.

```swift
func parseSummary(fileURL: URL) -> SessionSummary {
    // Read first 5 lines only — enough for metadata
    let head = readLines(fileURL, count: 5)
    return SessionSummary(
        id: extractSessionId(head),
        title: extractFirstUserMessage(head),
        startedAt: extractTimestamp(head),
        model: extractModel(head)
    )
}
```

### Phase 2 — Lazy Full Parse
Only parse the complete transcript when the user selects a session. Cache the 50 most recent.

### Phase 3 — Batch Loading
Load projects in groups of 5, updating the UI between batches for progressive rendering.

Shall I implement this?


---
