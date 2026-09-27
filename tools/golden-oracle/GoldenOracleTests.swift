// GoldenOracleTests.swift — the golden-output harness for the Swift reference app (task M1-50).
//
// This file is NEVER added to $POIROT. It lives here, in the Heimdall repo, and gets copied into
// a throwaway checkout of upstream Poirot (pinned SHA, ADR 0001) by the CI workflow (M1-51), which
// adds it as a new test target member alongside the untouched Poirot sources. It reads real Poirot
// loaders and services — never reimplements their logic — and writes their outputs as canonical
// JSON (sorted keys, camelCase, UTC ISO-8601 dates with fractional seconds) so Rust tests (M1-53)
// can compare byte-for-byte. See README.md in this folder for the copy-in procedure, the exact
// output layout, and every deliberate skip/deviation from a literal 1:1 port of ipc-contract.md §5.
//
// Given GOLDEN_FIXTURES (a directory holding fixture sets, each shaped `<set>/claude-home/**` +
// `<set>/claude.json`) and GOLDEN_OUT (an output directory), every test below walks the fixture
// sets it needs and writes one file per unit of output under `$GOLDEN_OUT/<set>/...`.
//
// Uses Swift Testing (`@Suite`/`@Test`), not XCTest — confirmed by checking: zero files anywhere
// in PoirotTests import XCTest; every real test in this project already uses Testing. `PoirotTests`
// is a separate module from `Poirot` (project.yml: a `bundle.unit-test` target depending on the
// `Poirot` target), so every Poirot type this file uses needs `@testable import Poirot` below —
// without it, none of Poirot's own (implicitly internal) types are visible here at all.
//
// ponytail: one flat file, not a mini-framework — this only ever runs once per CI job, by design
// (ADR 0001 point 4: retired entirely once parity is reached), so there is no second call site to
// justify splitting these helpers into their own module.
//
// Every private helper type/function below is explicitly `nonisolated` — required, not stylistic:
// `project.yml` sets `SWIFT_DEFAULT_ACTOR_ISOLATION: MainActor`, so anything left unmarked in this
// module becomes implicitly @MainActor-isolated, and every Poirot type these helpers wrap
// (`Session`, `Message`, `TokenUsage`, ...) is itself `nonisolated` for the same reason. The
// `GoldenOracleTests` suite struct itself is left unmarked (Swift Testing, unlike XCTestCase, has
// no fixed isolation expectation on its own suite/test declarations to conflict with).

import Foundation
@testable import Poirot
import Testing

// MARK: - Environment

nonisolated private enum Env {
    static var fixtures: String? { ProcessInfo.processInfo.environment["GOLDEN_FIXTURES"] }
    static var out: String? { ProcessInfo.processInfo.environment["GOLDEN_OUT"] }
}

/// One fixture set under `$GOLDEN_FIXTURES`. Every set in `fixtures/README.md`'s table that has a
/// `claude-home/` + `claude.json` pair (i.e. every set except `fixtures/golden/`, `fixtures/dto/`
/// and `fixtures/security/`, none of which this harness touches).
nonisolated private struct FixtureSet {
    let name: String
    let claudeHome: String // "<root>/<name>/claude-home"
    let projectsPath: String
    let historyFilePath: String
    let todosPath: String
    let debugPath: String
    let facetsPath: String
    let fileHistoryPath: String
    let statsCachePath: String

    init(root: String, name: String) {
        self.name = name
        let home = (root as NSString).appendingPathComponent(name).appending("/claude-home")
        claudeHome = home
        projectsPath = home.appending("/projects")
        historyFilePath = home.appending("/history.jsonl")
        todosPath = home.appending("/todos")
        debugPath = home.appending("/debug")
        facetsPath = home.appending("/usage-data/facets")
        fileHistoryPath = home.appending("/file-history")
        statsCachePath = home.appending("/stats-cache.json")
    }

    static func discover(root: String) -> [FixtureSet] {
        // Every named set `fixtures/README.md` documents with a `claude-home/` + `claude.json`
        // pair. Listed explicitly (not scanned) so a typo'd or missing set fails loudly via an
        // empty projects dump rather than silently skipping.
        ["synthetic", "screenshot", "windows", "linux"]
            .map { FixtureSet(root: root, name: $0) }
            .filter { FileManager.default.fileExists(atPath: $0.claudeHome) }
    }
}

// MARK: - Canonical JSON

// `nonisolated(unsafe)`, matching every ISO8601DateFormatter/DateFormatter global elsewhere in
// Poirot's own codebase (e.g. SessionLoader.swift, StatsComputer.swift): under Swift 6 strict
// concurrency, a plain `let` of a non-Sendable Foundation type still needs this to compile cleanly
// as global state. Read-only after initialization, so this is exactly as safe as Poirot's own use.
nonisolated(unsafe) private let isoFractional: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return f // defaults to UTC — ISO8601DateFormatter's timeZone is GMT unless set otherwise
}()

nonisolated private func iso(_ date: Date) -> String { isoFractional.string(from: date) }
nonisolated private func iso(_ date: Date?) -> String? { date.map(isoFractional.string(from:)) }

/// Encodes with sorted keys (canonical byte order) and unescaped slashes. Every "Golden*" wrapper
/// type below is written by hand rather than reusing a Poirot model's own `Codable` conformance,
/// because several on-disk models (`SessionFacets`, `SessionTodo.Status`) have snake_case
/// `CodingKeys` for their *disk* format — re-encoding through that conformance would leak
/// snake_case into what must be camelCase golden output.
nonisolated private func canonicalJSON<T: Encodable>(_ value: T) throws -> Data {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes, .prettyPrinted]
    return try encoder.encode(value)
}

nonisolated private func write(_ data: Data, to path: String) throws {
    let url = URL(fileURLWithPath: path)
    try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    try data.write(to: url)
}

nonisolated private func writeJSON<T: Encodable>(_ value: T, to path: String) throws {
    try write(try canonicalJSON(value), to: path)
}

nonisolated private func safeName(_ s: String) -> String {
    // File/directory names must not carry path separators a fixture's own data might contain.
    s.replacingOccurrences(of: "/", with: "_").replacingOccurrences(of: "\\", with: "_")
}

// MARK: - Golden wrapper types
//
// Every field name below matches docs/ipc-contract.md §5 exactly, except where a comment says
// otherwise. Fields ipc-contract lists that no Poirot model or loader actually computes (they are
// Rust-side-only aggregates — `pathSource`, `hasMemory`, `hasDebugLog`, `hasFileHistory`,
// `hasTodos`, `toolNames`, `skipped`, the `system` segments on a text block) are omitted rather
// than guessed at; see README.md's "Known gaps" section.

nonisolated private struct GoldenTokenUsage: Encodable {
    let input: Int, output: Int, cacheRead: Int, cacheCreation: Int
    init(_ u: TokenUsage?) {
        input = u?.input ?? 0; output = u?.output ?? 0
        cacheRead = u?.cacheRead ?? 0; cacheCreation = u?.cacheCreation ?? 0
    }
}

/// Flattens `ContentBlock` (a Swift enum) into one shape covering every case, matching
/// ipc-contract's tagged union but with `edit` inlined as `oldString`/`newString` rather than a
/// nested object — same information, simpler to encode from an `enum` case by case.
nonisolated private struct GoldenContentBlock: Encodable {
    let kind: String
    var id: String? = nil
    var text: String? = nil
    var toolUseId: String? = nil
    var name: String? = nil
    var inputPretty: String? = nil
    var command: String? = nil
    var filePath: String? = nil
    var oldString: String? = nil
    var newString: String? = nil
    var content: String? = nil
    var isError: Bool? = nil

    init(_ block: ContentBlock) {
        switch block {
        case let .text(text):
            kind = "text"; self.text = text
        case let .thinking(text):
            kind = "thinking"; self.text = text
        case let .toolUse(tool):
            kind = "toolUse"
            id = tool.id; toolUseId = tool.id; name = tool.name
            command = tool.command; filePath = tool.filePath
            oldString = tool.oldString; newString = tool.newString
            let sortedInput = tool.input.sorted { $0.key < $1.key }
            inputPretty = sortedInput.map { "\($0.key)=\($0.value)" }.joined(separator: "\n")
        case let .toolResult(result):
            kind = "toolResult"
            id = result.id; toolUseId = result.toolUseId
            content = result.content; isError = result.isError
        }
    }
}

nonisolated private struct GoldenMessage: Encodable {
    let id: String, role: String, timestamp: String?, model: String?
    let usage: GoldenTokenUsage?, blocks: [GoldenContentBlock]

    init(_ m: Message) {
        id = m.id
        role = m.role.rawValue // Message.Role: String, Codable already gives "user"/"assistant"/"system"
        timestamp = iso(m.timestamp)
        model = m.model
        usage = m.tokenUsage.map(GoldenTokenUsage.init)
        blocks = m.content.map(GoldenContentBlock.init)
    }
}

/// The header/summary tiers (`Session` built with `messages: []`) and the full-parse tier (built
/// with real `messages`) share this shape; `messageCount`/`blocks` distinguish them at a glance.
nonisolated private struct GoldenSession: Encodable {
    let id: String, projectPath: String, title: String?, preview: String?
    let startedAt: String, endedAt: String?, model: String?
    /// Always accurate at every tier — `Session.totalTokens` itself is a plain `Int` even at the
    /// full-parse tier (Poirot never retains a session-level input/output/cache breakdown).
    let totalTokens: Int
    /// The input/output/cache breakdown, summed from real per-message usage. Only available at
    /// the full-parse tier (`includeMessages: true`) — header and summary tiers build `Session`
    /// with `messages: []`, so there is nothing to sum a breakdown from at those tiers.
    let tokens: GoldenTokenUsage?
    let turnCount: Int
    let isSidechain: Bool, parentSessionId: String?, agentType: String?, agentDescription: String?
    let messageCount: Int
    let messages: [GoldenMessage]?

    init(_ s: Session, includeMessages: Bool) {
        id = s.id; projectPath = s.projectPath
        title = s.cachedTitle
        preview = s.cachedPreview
        startedAt = iso(s.startedAt); endedAt = iso(s.endedAt)
        model = s.model
        totalTokens = s.totalTokens
        turnCount = s.cachedTurnCount ?? s.messages.count { $0.role == .user }
        isSidechain = s.isSidechain; parentSessionId = s.parentSessionId
        agentType = s.agentType; agentDescription = s.agentDescription
        messageCount = s.messages.count
        if includeMessages {
            messages = s.messages.map(GoldenMessage.init)
            let usages = s.messages.compactMap(\.tokenUsage)
            tokens = GoldenTokenUsage(TokenUsage(
                input: usages.reduce(0) { $0 + $1.input },
                output: usages.reduce(0) { $0 + $1.output },
                cacheRead: usages.reduce(0) { $0 + $1.cacheRead },
                cacheCreation: usages.reduce(0) { $0 + $1.cacheCreation }
            ))
        } else {
            messages = nil
            tokens = nil
        }
    }
}

nonisolated private struct GoldenProjectSummary: Encodable {
    let id: String, name: String, path: String, sessionCount: Int, lastActivity: String?
}

nonisolated private struct GoldenContextUsage: Encodable {
    let current: Int, peak: Int, window: Int
    let inputTokens: Int, outputTokens: Int, cacheReadTokens: Int, cacheCreationTokens: Int
    let series: [Int]

    init(_ c: ContextUsage) {
        current = c.current; peak = c.peak; window = c.window
        inputTokens = c.inputTokens; outputTokens = c.outputTokens
        cacheReadTokens = c.cacheReadTokens; cacheCreationTokens = c.cacheCreationTokens
        series = c.series
    }
}

nonisolated private struct GoldenDiffLine: Encodable {
    let kind: String, oldLine: Int?, newLine: Int?, text: String
    init(_ l: DiffLine) {
        kind = l.kind == .context ? "context" : (l.kind == .added ? "add" : "remove")
        oldLine = l.oldLineNumber; newLine = l.newLineNumber; text = l.text
    }
}

nonisolated private struct GoldenDiffResult: Encodable {
    let lines: [GoldenDiffLine], truncated: Bool, maxLineNumber: Int
    init(_ lines: [DiffLine]) {
        self.lines = lines.map(GoldenDiffLine.init)
        truncated = false // LineDiff.swift never truncates — Rust's 5000-line cap has no Poirot equivalent
        maxLineNumber = lines.reduce(0) { max($0, $1.oldLineNumber ?? 0, $1.newLineNumber ?? 0) }
    }
}

nonisolated private struct GoldenTodoItem: Encodable {
    let content: String, activeForm: String, status: String
    init(_ t: SessionTodo) {
        content = t.content; activeForm = t.activeForm
        switch t.status {
        case .pending: status = "pending"
        case .inProgress: status = "inProgress"
        case .completed: status = "completed"
        }
    }
}

nonisolated private struct GoldenFacets: Encodable {
    let sessionId: String, underlyingGoal: String, goalCategories: [String: Int], outcome: String
    let userSatisfactionCounts: [String: Int], claudeHelpfulness: String, sessionType: String
    let frictionCounts: [String: Int], frictionDetail: String, primarySuccess: String, briefSummary: String
    init(_ f: SessionFacets) {
        sessionId = f.sessionId; underlyingGoal = f.underlyingGoal; goalCategories = f.goalCategories
        outcome = f.outcome; userSatisfactionCounts = f.userSatisfactionCounts
        claudeHelpfulness = f.claudeHelpfulness; sessionType = f.sessionType
        frictionCounts = f.frictionCounts; frictionDetail = f.frictionDetail
        primarySuccess = f.primarySuccess; briefSummary = f.briefSummary
    }
}

nonisolated private struct GoldenHistoryEntry: Encodable {
    let id: String, display: String, pastedContents: [String: String], timestamp: String, project: String
    init(_ e: HistoryEntry) {
        id = e.id; display = e.display; pastedContents = e.pastedContents
        timestamp = iso(e.timestamp); project = e.project
    }
}

nonisolated private struct GoldenDebugLogEntry: Encodable {
    let id: Int, timestamp: String, level: String, message: String
    init(_ e: DebugLogEntry) {
        id = e.id; timestamp = iso(e.timestamp)
        level = e.level.rawValue.lowercased() // Poirot: "DEBUG"/"WARN"/"ERROR" — contract wants lowercase
        message = e.message
    }
}

nonisolated private struct GoldenDebugLogPage: Encodable {
    let entries: [GoldenDebugLogEntry], offset: Int, total: Int
}

nonisolated private struct GoldenFileVersion: Encodable {
    let versionId: String, version: Int, backupTime: String // versionId ← backupFileName (README)
    init(_ v: FileVersion) { versionId = v.backupFileName; version = v.version; backupTime = iso(v.backupTime) }
}

nonisolated private struct GoldenFileHistoryEntry: Encodable {
    let fileName: String, versions: [GoldenFileVersion]
    init(_ e: FileHistoryEntry) { fileName = e.fileName; versions = e.versions.map(GoldenFileVersion.init) }
}

nonisolated private struct GoldenStats: Encodable {
    let computed: StatsCache?
    let cached: StatsCache?
    let latestSessionDay: String?
    let isStale: Bool? // only meaningful when `cached` is non-nil
    let chosenSource: String // "cache" | "computed" | "none" — what loadAnalytics would return
}

// MARK: - Session walking

nonisolated private struct SessionFile {
    let projectId: String // Project.id == the raw claude-home/projects/<dirName>
    let projectPath: String
    let sessionId: String
    let url: URL
    /// True for `agent-*.jsonl` files. `TranscriptParser.parse`/`parseSummary` default to
    /// `isSidechain: false`, which filters out every record an agent transcript's own lines mark
    /// `isSidechain: true` — `SessionLoader` always passes `true` explicitly for these files
    /// (`buildProject`'s `agentFiles` loop and `loadSubAgentSessions`), so this harness must too.
    let isSidechain: Bool
}

/// Every top-level (non-subagent) `.jsonl` transcript directly under a project directory —
/// deliberately not the nested `<session>/subagents/*.jsonl` files, which `discoverProjects()`
/// already covers as part of each project's own `sessions` array (see README's "Known gaps").
nonisolated private func topLevelSessionFiles(projectsPath: String) -> [SessionFile] {
    let fm = FileManager.default
    guard let projectDirs = try? fm.contentsOfDirectory(at: URL(fileURLWithPath: projectsPath), includingPropertiesForKeys: nil, options: [.skipsHiddenFiles]) else {
        return []
    }
    var result: [SessionFile] = []
    for dir in projectDirs {
        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: dir.path, isDirectory: &isDir), isDir.boolValue else { continue }
        let dirName = dir.lastPathComponent
        let projectPath = "/" + dirName.replacingOccurrences(of: "-", with: "/")
        guard let files = try? fm.contentsOfDirectory(at: dir, includingPropertiesForKeys: nil, options: [.skipsHiddenFiles]) else { continue }
        for file in files where file.pathExtension == "jsonl" {
            let sessionId = file.deletingPathExtension().lastPathComponent
            result.append(SessionFile(projectId: dirName, projectPath: projectPath, sessionId: sessionId, url: file, isSidechain: sessionId.hasPrefix("agent-")))
        }
    }
    return result
}

// MARK: - Tests

/// Skipped entirely (not failed) when `GOLDEN_FIXTURES`/`GOLDEN_OUT` are unset, via the
/// suite-level trait below — `init()` only ever runs for a test that trait already let through.
@Suite("Golden Oracle", .enabled(if: Env.fixtures != nil && Env.out != nil))
struct GoldenOracleTests {
    let outRoot: String
    let sets: [FixtureSet]

    init() throws {
        outRoot = Env.out!
        sets = FixtureSet.discover(root: Env.fixtures!)
        try #require(!sets.isEmpty, "no fixture set under \(Env.fixtures!) has a claude-home/ directory")
    }

    func outPath(_ set: FixtureSet, _ relative: String) -> String {
        (outRoot as NSString).appendingPathComponent(set.name).appendingPathComponent(relative)
    }

    /// AC1: "projects and summaries" — `SessionLoader.discoverProjects()`, which itself builds
    /// each session via `parseSummary` (the summary tier), including sub-agent sessions.
    @Test
    func projectsAndSummaries() throws {
        for set in sets {
            let loader = SessionLoader(claudeProjectsPath: set.projectsPath)
            let projects = (try? loader.discoverProjects()) ?? []

            let summaries = projects.map { p in
                GoldenProjectSummary(
                    id: p.id, name: p.name, path: p.path,
                    sessionCount: p.sessions.count,
                    lastActivity: p.sessions.map(\.startedAt).max().map(iso)
                )
            }
            try writeJSON(summaries, to: outPath(set, "projects.json"))

            for p in projects {
                let sessions = p.sessions.map { GoldenSession($0, includeMessages: false) }
                try writeJSON(sessions, to: outPath(set, "sessions/\(safeName(p.id))/summaries.json"))
            }
        }
    }

    /// AC1: "full parses" — `TranscriptParser.parse`, direct per top-level session file.
    @Test
    func fullParses() throws {
        for set in sets {
            let parser = TranscriptParser()
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                guard let session = parser.parse(fileURL: sf.url, projectPath: sf.projectPath, sessionId: sf.sessionId, indexStartedAt: nil, isSidechain: sf.isSidechain) else { continue }
                let golden = GoldenSession(session, includeMessages: true)
                try writeJSON(golden, to: outPath(set, "sessions/\(safeName(sf.projectId))/\(safeName(sf.sessionId))/full.json"))
            }
        }
    }

    /// AC1: "header ... tiers" — `TranscriptParser.parseHeader` (fast path, first ~50 lines only).
    @Test
    func headerTiers() throws {
        for set in sets {
            let parser = TranscriptParser()
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                guard let session = parser.parseHeader(fileURL: sf.url, projectPath: sf.projectPath, sessionId: sf.sessionId) else { continue }
                let golden = GoldenSession(session, includeMessages: false)
                try writeJSON(golden, to: outPath(set, "sessions/\(safeName(sf.projectId))/\(safeName(sf.sessionId))/header.json"))
            }
        }
    }

    /// AC1: "... and summary tiers" — `TranscriptParser.parseSummary` run directly (as opposed to
    /// via `discoverProjects()`, which is `projectsAndSummaries()`'s job); same tier, isolated per
    /// file so a full-parse/header/summary mismatch on one session is easy to bisect.
    @Test
    func summaryTiersDirect() throws {
        for set in sets {
            let parser = TranscriptParser()
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                guard let session = parser.parseSummary(fileURL: sf.url, projectPath: sf.projectPath, sessionId: sf.sessionId, indexStartedAt: nil, isSidechain: sf.isSidechain) else { continue }
                let golden = GoldenSession(session, includeMessages: false)
                try writeJSON(golden, to: outPath(set, "sessions/\(safeName(sf.projectId))/\(safeName(sf.sessionId))/summary.json"))
            }
        }
    }

    /// AC1: "stats (computed and cache-staleness verdict)".
    @Test
    func stats() throws {
        for set in sets {
            let computed = StatsComputer.compute(projectsPath: set.projectsPath)
            let cached = StatsCacheLoader.load(from: set.statsCachePath)
            let latestDay = StatsComputer.latestSessionDay(projectsPath: set.projectsPath)
            let isStale = cached.map { StatsCacheLoader.isStale($0, latestSessionDay: latestDay) }
            let chosen: String = {
                guard let cached else { return computed == nil ? "none" : "computed" }
                return (isStale == true) ? "computed" : "cache"
            }()
            let golden = GoldenStats(computed: computed, cached: cached, latestSessionDay: latestDay, isStale: isStale, chosenSource: chosen)
            try writeJSON(golden, to: outPath(set, "stats.json"))
        }
    }

    /// AC1: "history".
    @Test
    func history() throws {
        for set in sets {
            let entries = HistoryLoader(historyFilePath: set.historyFilePath).loadAll().map(GoldenHistoryEntry.init)
            try writeJSON(entries, to: outPath(set, "history.json"))
        }
    }

    /// AC1: "todos".
    @Test
    func todos() throws {
        for set in sets {
            let grouped = TodoLoader(claudeTodosPath: set.todosPath).loadAllTodos()
            let golden = grouped.mapValues { $0.map(GoldenTodoItem.init) }
            try writeJSON(golden, to: outPath(set, "todos.json"))
        }
    }

    /// AC1: "debug-log pages" — one page per session that has a log, offset 0, a limit large
    /// enough to capture the whole file (so this also doubles as the unpaged dump).
    @Test
    func debugLogPages() throws {
        for set in sets {
            let loader = DebugLogLoader(claudeDebugPath: set.debugPath)
            for sessionId in loader.allSessionIds() {
                let page = loader.loadEntries(for: sessionId, offset: 0, limit: 1_000_000)
                let golden = GoldenDebugLogPage(entries: page.entries.map(GoldenDebugLogEntry.init), offset: 0, total: page.totalCount)
                try writeJSON(golden, to: outPath(set, "debug/\(safeName(sessionId)).json"))
            }
        }
    }

    /// AC1: "facets".
    @Test
    func facets() throws {
        for set in sets {
            let all = FacetsLoader(claudeFacetsPath: set.facetsPath).loadAllFacets()
            let golden = all.mapValues(GoldenFacets.init)
            try writeJSON(golden, to: outPath(set, "facets.json"))
        }
    }

    /// AC1: "file history" — one dump per session that has any, keyed by the same session ids the
    /// full-parse/header/summary dumps use, so cross-referencing them is a plain path lookup.
    @Test
    func fileHistory() throws {
        for set in sets {
            let loader = FileHistoryLoader(claudeProjectsPath: set.projectsPath, claudeFileHistoryPath: set.fileHistoryPath)
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                let entries = loader.loadFileHistory(for: sf.sessionId, projectPath: sf.projectPath)
                guard !entries.isEmpty else { continue }
                try writeJSON(entries.map(GoldenFileHistoryEntry.init), to: outPath(set, "file-history/\(safeName(sf.sessionId)).json"))
            }
        }
    }

    /// AC1: "context usage" — `ContextUsage.from(_:)` over each session's full-parse messages.
    @Test
    func contextUsage() throws {
        for set in sets {
            let parser = TranscriptParser()
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                guard let session = parser.parse(fileURL: sf.url, projectPath: sf.projectPath, sessionId: sf.sessionId, indexStartedAt: nil, isSidechain: sf.isSidechain) else { continue }
                let usage = ContextUsage.from(session.messages)
                guard !usage.isEmpty else { continue }
                try writeJSON(GoldenContextUsage(usage), to: outPath(set, "sessions/\(safeName(sf.projectId))/\(safeName(sf.sessionId))/context.json"))
            }
        }
    }

    /// AC1: "export Markdown" — `SessionExporter.toMarkdown`, raw text (not JSON: there is no DTO
    /// shape for a Markdown document). README.md flags the one non-deterministic line this
    /// produces (the locale/timezone-formatted "**Date:**" header) and what the CI workflow must
    /// pin for this file to compare byte-for-byte across runs.
    @Test
    func exportMarkdown() throws {
        for set in sets {
            let parser = TranscriptParser()
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                guard let session = parser.parse(fileURL: sf.url, projectPath: sf.projectPath, sessionId: sf.sessionId, indexStartedAt: nil, isSidechain: sf.isSidechain) else { continue }
                let markdown = SessionExporter.toMarkdown(session)
                let path = outPath(set, "sessions/\(safeName(sf.projectId))/\(safeName(sf.sessionId))/export.md")
                try write(Data(markdown.utf8), to: path)
            }
        }
    }

    /// AC1: "line diffs" — real data, not synthetic strings: every consecutive pair of versions of
    /// every file `FileHistoryLoader` finds, diffed with `LineDiff.diff`.
    @Test
    func lineDiffs() throws {
        for set in sets {
            let loader = FileHistoryLoader(claudeProjectsPath: set.projectsPath, claudeFileHistoryPath: set.fileHistoryPath)
            for sf in topLevelSessionFiles(projectsPath: set.projectsPath) {
                let entries = loader.loadFileHistory(for: sf.sessionId, projectPath: sf.projectPath)
                for entry in entries where entry.versions.count >= 2 {
                    for i in 0 ..< (entry.versions.count - 1) {
                        let (a, b) = (entry.versions[i], entry.versions[i + 1])
                        guard let oldText = loader.loadFileContent(for: sf.sessionId, backupFileName: a.backupFileName),
                              let newText = loader.loadFileContent(for: sf.sessionId, backupFileName: b.backupFileName)
                        else { continue }
                        let diff = GoldenDiffResult(LineDiff.diff(old: oldText, new: newText))
                        let name = "\(safeName(entry.fileName))-v\(a.version)-v\(b.version).json"
                        try writeJSON(diff, to: outPath(set, "diffs/\(safeName(sf.sessionId))/\(name)"))
                    }
                }
            }
        }
    }
}
