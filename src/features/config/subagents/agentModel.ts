// What the sub-agent form and page share (M3-49): the catalogue the form offers, the slug the
// writer will derive, and the mapping from the read API's `ConfigItemDetail` to the form's model.
//
// The tool lists and categories are `SubAgent::KNOWN_TOOLS` / `TOOL_CATEGORIES`
// (crates/heimdall-core/src/models/config/sub_agent.rs), restated here because nothing exposes
// them over IPC -- the same reasoning HookForm.vue gives for its event catalogue.
import type { AgentColor, AgentInput, AgentMemory, ConfigItemDetail } from '@/ipc/bindings';

export const KNOWN_TOOLS = ['Agent', 'AskUserQuestion', 'Bash', 'Edit', 'Glob', 'Grep', 'NotebookEdit', 'Read', 'TodoWrite', 'WebFetch', 'WebSearch', 'Write'] as const;

export const TOOL_CATEGORIES: { id: string; name: string; tools: readonly string[] }[] = [
  { id: 'readonly', name: 'Read-only tools', tools: ['Glob', 'Grep', 'Read', 'WebFetch', 'WebSearch'] },
  { id: 'edit', name: 'Edit tools', tools: ['Edit', 'Write', 'NotebookEdit'] },
  { id: 'execution', name: 'Execution tools', tools: ['Bash', 'Agent'] },
  { id: 'other', name: 'Other tools', tools: ['AskUserQuestion', 'TodoWrite'] },
];

/** SubAgentFormView.swift:21's own default for a new agent. */
export const DEFAULT_TOOLS = ['Glob', 'Grep', 'Read', 'Bash'];

export const MODELS: { value: string; label: string }[] = [
  { value: 'opus', label: 'Opus' },
  { value: 'sonnet', label: 'Sonnet' },
  { value: 'haiku', label: 'Haiku' },
];

export const COLORS: { value: AgentColor; label: string; fill: string }[] = [
  // Complete literal class strings: Tailwind only generates CSS for class names it can find in the
  // source text, so these cannot be built from an interpolated colour name.
  { value: 'orange', label: 'Orange', fill: 'bg-(--color-orange)' },
  { value: 'red', label: 'Red', fill: 'bg-(--color-red)' },
  { value: 'blue', label: 'Blue', fill: 'bg-(--color-blue)' },
  { value: 'green', label: 'Green', fill: 'bg-(--color-green)' },
  { value: 'purple', label: 'Purple', fill: 'bg-(--color-purple)' },
  { value: 'teal', label: 'Teal', fill: 'bg-(--color-teal)' },
];

export const MEMORIES: { value: AgentMemory; label: string }[] = [
  { value: 'user', label: 'Enable (~/.claude/agent-memory/)' },
  { value: 'none', label: 'None (no persistent memory)' },
];

/** What the form edits. `null` for model/colour/memory means "not set on this agent": the form
 * leaves it null rather than inventing a value, because Swift's own defaults (`?? "sonnet"`,
 * `?? .orange`, `?? .user`) silently pinned a model, added a colour and switched persistent memory
 * ON for any agent that had none of them, on every edit. Only a NEW agent gets those defaults. */
export interface AgentFormModel {
  /** The file stem being edited (`save_agent`'s `originalSlug`); `null` when creating. */
  slug: string | null;
  name: string;
  description: string;
  tools: string[];
  model: string | null;
  color: AgentColor | null;
  memory: AgentMemory | null;
  prompt: string;
}

export function newAgentModel(): AgentFormModel {
  return { slug: null, name: '', description: '', tools: [...DEFAULT_TOOLS], model: 'sonnet', color: 'orange', memory: 'user', prompt: '' };
}

/** `agents::agent_slug` (files.rs), the filename the writer derives: lowercased, spaces to hyphens,
 * then anything that is not a letter, a number or a hyphen stripped. Empty for a name with none. */
export function agentSlug(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/ /g, '-')
    .replace(/[^\p{Alphabetic}\p{N}-]/gu, '');
}

/** The filename a save will actually write: `unique_slug` (files.rs) never overwrites another
 * agent -- a clash gets `-2`, `-3`, ... -- and an edit that keeps its slug overwrites in place. */
export function predictSlug(name: string, originalSlug: string | null, existingSlugs: readonly string[]): string {
  const base = agentSlug(name);
  if (base === '') return '';
  if (originalSlug !== null && base === originalSlug) return base;
  let candidate = base;
  for (let n = 2; existingSlugs.includes(candidate); n++) candidate = `${base}-${n}`;
  return candidate;
}

const COLOR_VALUES = new Set<string>(COLORS.map((c) => c.value));

/** `agent_summary` (config_read.rs) flattens an agent into `meta`: `tools` comma-joined, `color` as
 * its Debug name ("Orange"), `memory` as the lowercase wire value, `model` raw. A colour or memory
 * the editor's own enums cannot represent never reaches here as anything but absent, which is why
 * `null` ("not set, leave it alone") is a real state of the form. */
export function agentFromDetail(detail: ConfigItemDetail): AgentFormModel {
  const color = detail.meta.color?.toLowerCase();
  const memory = detail.meta.memory;
  return {
    slug: detail.ref.slug,
    name: detail.name,
    description: detail.description,
    tools: detail.meta.tools ? detail.meta.tools.split(',').map((t) => t.trim()).filter(Boolean) : [],
    model: detail.meta.model ?? null,
    color: color && COLOR_VALUES.has(color) ? (color as AgentColor) : null,
    memory: memory === 'user' || memory === 'none' ? memory : null,
    prompt: detail.body,
  };
}

/** `SubAgentsListView.duplicateAgent`: the same agent saved as a new one named "<name> Copy". */
export function duplicateOf(source: AgentFormModel): AgentInput {
  return toInput({ ...source, slug: null, name: `${source.name} Copy` });
}

/** The only place a form model becomes the `save_agent` payload. `icon` is fixed: custom agents
 * cannot pick one (SubAgentFormView.swift, spec §9.2). An empty prompt is `null`, as in Swift. */
export function toInput(model: AgentFormModel): AgentInput {
  return {
    name: model.name.trim(),
    icon: 'person.crop.circle.badge.plus',
    description: model.description.trim(),
    tools: model.tools,
    model: model.model,
    color: model.color,
    prompt: model.prompt === '' ? null : model.prompt,
    memory: model.memory,
  };
}
