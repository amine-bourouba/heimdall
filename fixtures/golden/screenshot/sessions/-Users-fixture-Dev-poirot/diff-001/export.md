# Refactor the TranscriptParser to use async/await instead of completion handlers

**Project:** poirot
**Model:** claude-opus-4-6
**Date:** September 20, 2026 at 11:25 AM
**Turns:** 2
**Tokens:** 2.9k

---

## Turn 1 — User
*11:25:00*

Refactor the TranscriptParser to use async/await instead of completion handlers


---

## Turn 1 — Assistant
*11:26:00*

I'll convert the completion-based API to async/await for cleaner call sites.

### Tool: Edit — `Poirot/Sources/Services/TranscriptParser.swift`
```diff
- func parse(
-     fileURL: URL,
-     projectPath: String,
-     completion: @escaping (Session?) -> Void
- ) {
-     DispatchQueue.global(qos: .userInitiated).async {
-         let session = self.parseSync(fileURL: fileURL, projectPath: projectPath)
-         DispatchQueue.main.async {
-             completion(session)
-         }
-     }
- }
+ func parse(
+     fileURL: URL,
+     projectPath: String,
+     sessionId: String,
+     indexStartedAt: Date?
+ ) -> Session? {
+     parseSync(
+         fileURL: fileURL,
+         projectPath: projectPath,
+         sessionId: sessionId,
+         indexStartedAt: indexStartedAt
+     )
+ }
```


---

## Turn 2 — User
*11:26:00*

**Output:**
```
Successfully edited Poirot/Sources/Services/TranscriptParser.swift
```


---
