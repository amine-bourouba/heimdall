import { describe, expect, it } from 'vitest';
import { navigationItems } from './navigationItems';

// Ported from PoirotTests/Providers/NavigationItemTests.swift. requiredCapability's own tests
// (skills_requiresSkillsCapability etc.) are not ported: that field exists in the Swift reference app only
// because of its multi-provider abstraction, which ADR 0008 removes (one implementation only,
// user decision 3) -- there is nothing left to assert once every item is unconditionally offered.
describe('navigationItems (NavigationItemTests)', () => {
  it('has fourteen items, in the real NavigationItem.allItems order', () => {
    expect(navigationItems).toHaveLength(14);
    expect(navigationItems.map((item) => item.id)).toEqual([
      'sessions',
      'analytics',
      'todos',
      'history',
      'commands',
      'skills',
      'plans',
      'memory',
      'hooks',
      'mcpServers',
      'plugins',
      'outputStyles',
      'models',
      'subAgents',
    ]);
  });

  it('history has the correct icon', () => {
    const history = navigationItems.find((item) => item.id === 'history')!;
    expect(history.icon).toBe('clock.arrow.circlepath');
  });

  it('analytics has the correct icon', () => {
    const analytics = navigationItems.find((item) => item.id === 'analytics')!;
    expect(analytics.icon).toBe('chart.xyaxis.line');
  });

  it('memory has the correct icon', () => {
    const memory = navigationItems.find((item) => item.id === 'memory')!;
    expect(memory.icon).toBe('brain.head.profile.fill');
  });

  it('every icon has a real Lucide mapping (M2-04)', async () => {
    const { iconMap } = await import('@/components/iconMap');
    for (const item of navigationItems) {
      expect(iconMap[item.icon], `no iconMap entry for "${item.icon}" (id: ${item.id})`).toBeDefined();
    }
  });

  it('labels match PoirotTheme.swift/NavigationItem.swift', () => {
    const labelById = Object.fromEntries(navigationItems.map((item) => [item.id, item.label]));
    expect(labelById).toMatchObject({
      sessions: 'Sessions',
      analytics: 'Analytics',
      todos: 'TODOs',
      history: 'History',
      commands: 'Commands',
      skills: 'Skills',
      plans: 'Plans',
      memory: 'Memory',
      hooks: 'Hooks',
      mcpServers: 'MCP Servers',
      plugins: 'Plugins',
      outputStyles: 'Output Styles',
      models: 'Models',
      subAgents: 'Sub-agents',
    });
  });

  // M2-15, M3-02, M3-03, M3-04 and M3-05 are, so far, the only feature tasks to flip their own
  // item's flag (`session/index.ts`'s, `history/index.ts`'s, `plans/index.ts`'s,
  // `memory/index.ts`'s and `todos/index.ts`'s own `available = true`) -- every item whose owning
  // feature hasn't landed yet still starts false.
  it('every item without a landed feature yet starts unavailable', () => {
    const landed = new Set(['sessions', 'history', 'plans', 'memory', 'todos', 'commands', 'skills', 'outputStyles', 'hooks', 'plugins', 'models', 'mcpServers', 'subAgents']);
    const stillUnbuilt = navigationItems.filter((item) => !landed.has(item.id));
    expect(stillUnbuilt.every((item) => item.available === false)).toBe(true);
  });

  it('sessions is available now that M2-15 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'sessions')?.available).toBe(true);
  });

  it('plans is available now that M3-03 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'plans')?.available).toBe(true);
  });

  it('memory is available now that M3-04 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'memory')?.available).toBe(true);
  });

  it('todos is available now that M3-05 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'todos')?.available).toBe(true);
  });

  it('history is available now that M3-02 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'history')?.available).toBe(true);
  });

  it('commands is available now that M3-12 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'commands')?.available).toBe(true);
  });

  it('skills is available now that M3-12 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'skills')?.available).toBe(true);
  });

  it('outputStyles is available now that M3-12 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'outputStyles')?.available).toBe(true);
  });

  it('hooks is available now that M3-13 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'hooks')?.available).toBe(true);
  });

  it('plugins is available now that M3-13 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'plugins')?.available).toBe(true);
  });

  it('models is available now that M3-13 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'models')?.available).toBe(true);
  });

  it('mcpServers is available now that M3-14 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'mcpServers')?.available).toBe(true);
  });

  it('subAgents is available now that M3-14 has built it', () => {
    expect(navigationItems.find((item) => item.id === 'subAgents')?.available).toBe(true);
  });
});
