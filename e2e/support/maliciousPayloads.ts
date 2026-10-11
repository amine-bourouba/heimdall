// The committed `fixtures/security/malicious-claude-home/` tree and `e2e/security/malicious-
// transcript.spec.ts`'s own mocked session both need the M2-25 XSS corpus
// (`fixtures/security/xss-payloads.json`) distributed across the six locations ADR 0010's
// acceptance test list names (user text, assistant text, tool output, thinking, a plan, a memory
// file). Round-robin by array index, rather than repeating the whole corpus six times over, so
// the union across all six locations covers every payload exactly once -- this one function is
// the single source of truth both the (hand-generated, once) fixture tree and the spec file
// agree on, so they can never silently drift apart.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

interface Payload {
  id: string;
  category: string;
  description: string;
  markdown: string;
}

const payloadsPath = fileURLToPath(new URL('../../fixtures/security/xss-payloads.json', import.meta.url));
const payloads: Payload[] = JSON.parse(readFileSync(payloadsPath, 'utf-8'));

export const MALICIOUS_LOCATIONS = ['user', 'assistant', 'tool-output', 'thinking', 'plan', 'memory'] as const;
export type MaliciousLocation = (typeof MALICIOUS_LOCATIONS)[number];

export function maliciousPayloadsFor(location: MaliciousLocation): string[] {
  const index = MALICIOUS_LOCATIONS.indexOf(location);
  return payloads.filter((_, i) => i % MALICIOUS_LOCATIONS.length === index).map((p) => p.markdown);
}

export function joinedMaliciousContent(location: MaliciousLocation): string {
  return maliciousPayloadsFor(location).join('\n\n');
}
