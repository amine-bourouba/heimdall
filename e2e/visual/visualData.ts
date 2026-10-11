// Ported verbatim from `PoirotTests/Mocks/ScreenshotData.swift`'s own tool/result/diff/thinking
// fixtures -- the real, hand-built data every Swift screenshot test in `ToolBlocks`/`ThinkingBlocks`/
// `EditDiff` renders. Swift's `ToolUse.input: [String: String]` is reshaped into Heimdall's own
// pre-processed contract (`command`/`filePath`/`inputPretty`/`edit`, Rust's own job in the real
// app) by hand here, once, the same reshaping `ToolBlock.vue` itself already does for `Bash`/
// file-path tools.
import type { ContentBlock, ToolResult, ToolUse } from '../../src/ipc/bindings';

function tool(overrides: Partial<ToolUse>): ToolUse {
  return { id: 't', toolUseId: 'tu', name: '', inputPretty: '{}', command: null, filePath: null, edit: null, ...overrides };
}

function result(overrides: Partial<ToolResult>): ToolResult {
  return { id: 'r', toolUseId: 'tu', content: '', isError: false, ...overrides };
}

export const TOOLS: Record<string, { tool: ToolUse; result: ToolResult }> = {
  read: {
    tool: tool({ id: 'standalone-read', toolUseId: 'standalone-read', name: 'Read', filePath: 'Poirot/Sources/App/ContentView.swift' }),
    result: result({
      id: 'standalone-read-result',
      toolUseId: 'standalone-read',
      content:
        'import SwiftUI\n\nstruct ContentView: View {\n    @Environment(AppState.self) private var appState\n    @Environment(\\.provider) private var provider\n    @State private var sidebarVisibility: NavigationSplitViewVisibility = .automatic\n\n    var body: some View {\n        NavigationSplitView(columnVisibility: $sidebarVisibility) {\n            SidebarView()\n                .navigationSplitViewColumnWidth(min: 220, ideal: 260, max: 300)\n        } detail: {\n            detailView\n        }\n        .navigationSplitViewStyle(.prominentDetail)\n    }\n}',
    }),
  },
  write: {
    tool: tool({
      id: 'standalone-write',
      toolUseId: 'standalone-write',
      name: 'Write',
      filePath: 'Poirot/Sources/Services/FileWatcher.swift',
      inputPretty: '{\n  "content": "import Foundation\\n\\nfinal class FileWatcher..."\n}',
    }),
    result: result({ id: 'standalone-write-result', toolUseId: 'standalone-write', content: 'Successfully wrote to Poirot/Sources/Services/FileWatcher.swift' }),
  },
  edit: {
    tool: tool({
      id: 'standalone-edit',
      toolUseId: 'standalone-edit',
      name: 'Edit',
      filePath: 'Poirot/Sources/Theme/PoirotTheme.swift',
      edit: { oldString: 'static let accent = Color.orange', newString: 'static let accent = Color(hex: "#E8A642")' },
    }),
    result: result({ id: 'standalone-edit-result', toolUseId: 'standalone-edit', content: 'Successfully edited Poirot/Sources/Theme/PoirotTheme.swift' }),
  },
  bash: {
    tool: tool({ id: 'standalone-bash', toolUseId: 'standalone-bash', name: 'Bash', command: "xcodebuild build -scheme Poirot -destination 'platform=macOS' 2>&1 | tail -3" }),
    result: result({
      id: 'standalone-bash-result',
      toolUseId: 'standalone-bash',
      content: 'Ld /Users/leo/Library/Developer/Xcode/DerivedData/Poirot-abc123/Build/Products/Debug/Poirot.app\nGenerateDSYMFile Poirot.app.dSYM\n** BUILD SUCCEEDED **',
    }),
  },
  bashError: {
    tool: tool({ id: 'standalone-bash-error', toolUseId: 'standalone-bash-error', name: 'Bash', command: 'swift test --filter TranscriptParserTests' }),
    result: result({
      id: 'standalone-bash-error-result',
      toolUseId: 'standalone-bash-error',
      isError: true,
      content:
        "Test Suite 'TranscriptParserTests' started\nTest Case 'testParseMalformedJSON' started\n/Tests/TranscriptParserTests.swift:42: error: testParseMalformedJSON\n    #expect(session != nil) failed: session is nil\nTest Case 'testParseMalformedJSON' failed (0.003 seconds)\nTest Suite 'TranscriptParserTests' failed\n     Executed 3 tests, with 1 failure in 0.018 (0.020) seconds",
    }),
  },
  glob: {
    tool: tool({ id: 'standalone-glob', toolUseId: 'standalone-glob', name: 'Glob', inputPretty: '{\n  "pattern": "**/*.swift",\n  "path": "Poirot/Sources"\n}' }),
    result: result({
      id: 'standalone-glob-result',
      toolUseId: 'standalone-glob',
      content:
        'Poirot/Sources/App/AppState.swift\nPoirot/Sources/App/ContentView.swift\nPoirot/Sources/App/PoirotApp.swift\nPoirot/Sources/Models/Session.swift\nPoirot/Sources/Models/Message.swift\nPoirot/Sources/Models/ToolUse.swift\nPoirot/Sources/Services/TranscriptParser.swift\nPoirot/Sources/Services/SessionLoader.swift\nPoirot/Sources/Services/FileWatcher.swift\nPoirot/Sources/Theme/PoirotTheme.swift\nPoirot/Sources/Views/Components/SidebarView.swift\nPoirot/Sources/Views/Components/ToastOverlay.swift\nPoirot/Sources/Views/Session/SessionDetailView.swift\nPoirot/Sources/Views/Session/ToolBlockView.swift',
    }),
  },
  grep: {
    tool: tool({ id: 'standalone-grep', toolUseId: 'standalone-grep', name: 'Grep', inputPretty: '{\n  "pattern": "@Observable",\n  "path": "Poirot/Sources"\n}' }),
    result: result({
      id: 'standalone-grep-result',
      toolUseId: 'standalone-grep',
      content: 'Poirot/Sources/App/AppState.swift:4:@Observable\nPoirot/Sources/App/AppState.swift:5:final class AppState {',
    }),
  },
  task: {
    tool: tool({
      id: 'standalone-task',
      toolUseId: 'standalone-task',
      name: 'Task',
      inputPretty: '{\n  "description": "Run snapshot tests",\n  "prompt": "Execute all snapshot tests and report any failures or new baselines needed"\n}',
    }),
    result: result({
      id: 'standalone-task-result',
      toolUseId: 'standalone-task',
      content: 'Snapshot test results:\n- 68 tests executed, 68 passed\n- 0 failures, 0 new baselines\n- Total duration: 4.82s\n\nAll snapshots match their reference images.',
    }),
  },
  unknown: {
    tool: tool({
      id: 'standalone-unknown',
      toolUseId: 'standalone-unknown',
      name: 'mcp__sentry__search_issues',
      inputPretty: '{\n  "query": "TranscriptParser crash",\n  "project": "poirot"\n}',
    }),
    result: result({
      id: 'standalone-unknown-result',
      toolUseId: 'standalone-unknown',
      content: "Found 2 issues matching 'TranscriptParser crash':\n1. POIROT-42: Crash on malformed JSONL (resolved)\n2. POIROT-58: OOM on large session files (open)",
    }),
  },
  noOutput: {
    tool: tool({ id: 'standalone-no-output', toolUseId: 'standalone-no-output', name: 'Bash', command: 'mkdir -p Poirot/Sources/Services' }),
    result: result({ id: 'standalone-no-output-result', toolUseId: 'standalone-no-output', content: '' }),
  },
  longContent: {
    tool: tool({ id: 'standalone-long', toolUseId: 'standalone-long', name: 'Bash', command: "xcodebuild test -scheme Poirot -destination 'platform=macOS' 2>&1" }),
    result: result({
      id: 'standalone-long-result',
      toolUseId: 'standalone-long',
      content: [
        "Test Suite 'All tests' started at 2026-02-20 14:25:00\n",
        ...Array.from({ length: 68 }, (_, i) => {
          const n = i + 1;
          const dur = ((10 + (n * 13) % 400) / 1000).toFixed(3);
          return `  Test Case 'testSnapshot_${String(n).padStart(3, '0')}' passed (${dur} seconds)`;
        }),
        "\nTest Suite 'All tests' passed",
        '  Executed 68 tests, with 0 failures in 4.821 (5.003) seconds',
      ].join('\n'),
    }),
  },
};

export const DIFFS: Record<string, { oldString: string; newString: string; filePath: string }> = {
  simple: { oldString: 'static let accent = Color.orange', newString: 'static let accent = Color(hex: "#E8A642")', filePath: 'Poirot/Sources/Theme/PoirotTheme.swift' },
  multiLine: {
    oldString:
      'func parse(\n    fileURL: URL,\n    projectPath: String,\n    completion: @escaping (Session?) -> Void\n) {\n    DispatchQueue.global(qos: .userInitiated).async {\n        let session = self.parseSync(fileURL: fileURL, projectPath: projectPath)\n        DispatchQueue.main.async {\n            completion(session)\n        }\n    }\n}',
    newString:
      'func parse(\n    fileURL: URL,\n    projectPath: String,\n    sessionId: String,\n    indexStartedAt: Date?\n) -> Session? {\n    parseSync(\n        fileURL: fileURL,\n        projectPath: projectPath,\n        sessionId: sessionId,\n        indexStartedAt: indexStartedAt\n    )\n}',
    filePath: 'Poirot/Sources/Services/SessionLoader.swift',
  },
  addOnly: {
    oldString: 'struct Toast {\n    let message: AttributedString\n    let icon: String\n}',
    newString: 'struct Toast {\n    let message: AttributedString\n    let icon: String\n    let style: Style\n    let url: URL?\n\n    enum Style { case success, error, info }\n}',
    filePath: 'Poirot/Sources/Models/Config.swift',
  },
  removeOnly: {
    oldString:
      'struct SessionLoader {\n    let cache: NSCache<NSString, NSData>\n    let queue: DispatchQueue\n    let semaphore: DispatchSemaphore\n\n    func loadAll() {\n        semaphore.wait()\n        defer { semaphore.signal() }\n        // synchronous full parse\n    }\n}',
    newString: 'struct SessionLoader {\n    func loadAll() async {\n        // async full parse\n    }\n}',
    filePath: 'Poirot/Sources/Services/LegacyLoader.swift',
  },
};

export const THINKING: Record<string, string> = {
  collapsed: "Analyzing the user's request for performance optimization...",
  expanded:
    'The user is asking about performance optimization for session loading. Let me think about this carefully.\n\nCurrent flow:\n1. SessionLoader scans ~/.claude/projects/ directories\n2. For each project, it reads ALL session JSONL files\n3. Each file is fully parsed into Session/Message models\n4. All sessions are loaded into memory\n\nBottlenecks:\n- Full JSONL parsing is expensive — each file can be several MB\n- Sequential loading — no parallelism\n- No caching — reloads everything on each app launch\n\nI think lazy loading combined with batch loading would be the biggest win.',
  longContent: ['Analyzing the codebase structure:\n', ...Array.from({ length: 80 }, (_, i) => `${i + 1}. Checking module dependency_${String(i + 1).padStart(3, '0')}: resolved`)].join('\n'),
  short: 'Quick check — yes, this is correct.',
};

/** One block per tool, in a stable order -- the "all tools" conversation used by the SessionDetail/
 * FullViews suites (and the 3-themes x 2-font-scales conversation view) that need a real, full
 * session rather than one isolated component. */
export function allToolsBlocks(): ContentBlock[] {
  return Object.values(TOOLS).flatMap(({ tool: t, result: r }): ContentBlock[] => [
    { kind: 'toolUse', ...t },
    { kind: 'toolResult', ...r },
  ]);
}
