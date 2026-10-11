// Generates a synthetic session transcript of 10,000+ content blocks, in Heimdall's own `Message`
// DTO shape (not raw Claude JSONL -- the frontend never sees that format, `heimdall-core` already
// converts it) -- port-plan.md §9's "Transcript scrolling at 10,000+ blocks" budget (M2-28). Not
// committed as a fixture (fixtures/README.md's own "byte-exact, hand-built" rule is for small,
// stable fixtures; a 10k+-block transcript is neither small nor meant to be hand-edited) --
// `e2e/perf/transcript-scroll.spec.ts` calls `generateLargeTranscriptMessages()` directly at test
// time instead.
//
// Every 5th message is a Bash tool_use + its tool_result, matching a real coding session's own
// mix of plain conversation and tool activity (not just 10,000 identical text bubbles) -- the
// virtualizer's own row-height variance (and `ToolBlock`'s own extra DOM weight) is part of what
// a scroll-smoothness budget is meant to catch, not something a uniform fixture would exercise.

import { writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const FILLER =
  'Looking at the current implementation, the bottleneck is in how the parser walks the AST for each top-level declaration before resolving imports and re-exporting the public surface.';

/** @param {number} minBlockCount @returns {unknown[]} Heimdall `Message[]` DTOs */
export function generateLargeTranscriptMessages(minBlockCount = 10_000) {
  const messages = [];
  const base = Date.parse('2026-01-01T00:00:00.000Z');
  let blockCount = 0;
  let i = 0;

  while (blockCount < minBlockCount) {
    const role = i % 2 === 0 ? 'user' : 'assistant';
    const timestamp = new Date(base + i * 1000).toISOString();
    const blocks = [{ kind: 'text', id: `m${i}b0`, text: `Message ${i}: ${FILLER}`, system: null }];
    blockCount += 1;

    if (role === 'assistant' && i % 5 === 0) {
      const toolUseId = `tu${i}`;
      blocks.push({ kind: 'toolUse', id: `m${i}b1`, toolUseId, name: 'Bash', inputPretty: '{}', command: `echo step-${i}`, filePath: null, edit: null });
      blocks.push({ kind: 'toolResult', id: `m${i}b2`, toolUseId, content: `step-${i} ok`, isError: false });
      blockCount += 2;
    }

    messages.push({
      id: `m${i}`,
      role,
      timestamp,
      model: role === 'assistant' ? 'claude-opus-4-6' : null,
      usage: role === 'assistant' ? { input: 50, output: 20, cacheRead: 0, cacheCreation: 0 } : null,
      blocks,
    });
    i += 1;
  }

  return messages;
}

async function main() {
  const outPath = process.argv[2];
  const minBlockCount = Number(process.argv[3] ?? 10_000);
  const messages = generateLargeTranscriptMessages(minBlockCount);
  const totalBlocks = messages.reduce((sum, m) => sum + m.blocks.length, 0);

  if (outPath) {
    writeFileSync(outPath, JSON.stringify(messages));
    console.log(`Wrote ${messages.length} messages (${totalBlocks} blocks) to ${outPath}`);
  } else {
    console.log(`Generated ${messages.length} messages (${totalBlocks} blocks); pass an output path to write it to disk.`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
