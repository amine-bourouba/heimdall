// The sidebar's own item catalog (ported from NavigationItem.swift's `allItems`, 14 entries,
// exact order). `label`/`available` are each feature's own index.ts (never duplicated here);
// `icon` is an SF Symbol key straight into src/components/iconMap.ts (M2-04), so AppIcon renders
// it with no translation step. `requiredCapability` has no port: it exists in the Swift reference app only
// because of its multi-provider abstraction (ADR 0008 removes that -- one implementation only,
// user decision 3), so every item here is unconditionally offered; "available" (this task) is
// the only gate, and it is about whether the *feature itself* has been built yet, not about
// capability.

import * as analytics from '@/features/analytics';
import * as commandsFeature from '@/features/config/commands';
import * as hooksFeature from '@/features/config/hooks';
import * as mcpFeature from '@/features/config/mcp';
import * as modelsFeature from '@/features/config/models';
import * as outputStylesFeature from '@/features/config/output-styles';
import * as pluginsFeature from '@/features/config/plugins';
import * as skillsFeature from '@/features/config/skills';
import * as subAgentsFeature from '@/features/config/subagents';
import * as history from '@/features/history';
import * as memory from '@/features/memory';
import * as plans from '@/features/plans';
import * as sessions from '@/features/session';
import * as todos from '@/features/todos';
import type { RouteLocationRaw } from 'vue-router';

export interface NavigationItem {
  id: string;
  label: string;
  icon: string;
  available: boolean;
  to: RouteLocationRaw;
}

export const navigationItems: NavigationItem[] = [
  { id: 'sessions', label: sessions.label, icon: 'rectangle.stack.fill', available: sessions.available, to: { name: 'sessions' } },
  { id: 'analytics', label: analytics.label, icon: 'chart.xyaxis.line', available: analytics.available, to: { name: 'analytics' } },
  { id: 'todos', label: todos.label, icon: 'checklist', available: todos.available, to: { name: 'todos' } },
  { id: 'history', label: history.label, icon: 'clock.arrow.circlepath', available: history.available, to: { name: 'history' } },
  {
    id: 'commands',
    label: commandsFeature.label,
    icon: 'apple.terminal.fill',
    available: commandsFeature.available,
    to: { name: 'config-list', params: { kind: 'command' } },
  },
  {
    id: 'skills',
    label: skillsFeature.label,
    icon: 'bolt.fill',
    available: skillsFeature.available,
    to: { name: 'config-list', params: { kind: 'skill' } },
  },
  { id: 'plans', label: plans.label, icon: 'list.bullet.clipboard.fill', available: plans.available, to: { name: 'plans' } },
  { id: 'memory', label: memory.label, icon: 'brain.head.profile.fill', available: memory.available, to: { name: 'memory' } },
  {
    id: 'hooks',
    label: hooksFeature.label,
    icon: 'arrow.triangle.branch',
    available: hooksFeature.available,
    to: { name: 'hooks' },
  },
  {
    id: 'mcpServers',
    label: mcpFeature.label,
    icon: 'powerplug.fill',
    available: mcpFeature.available,
    to: { name: 'mcp' },
  },
  {
    id: 'plugins',
    label: pluginsFeature.label,
    icon: 'puzzlepiece.fill',
    available: pluginsFeature.available,
    to: { name: 'plugins' },
  },
  {
    id: 'outputStyles',
    label: outputStylesFeature.label,
    icon: 'speaker.wave.3.fill',
    available: outputStylesFeature.available,
    to: { name: 'config-list', params: { kind: 'outputStyle' } },
  },
  {
    id: 'models',
    label: modelsFeature.label,
    icon: 'brain.fill',
    available: modelsFeature.available,
    to: { name: 'models' },
  },
  {
    id: 'subAgents',
    label: subAgentsFeature.label,
    icon: 'person.2.fill',
    available: subAgentsFeature.available,
    to: { name: 'config-list', params: { kind: 'agent' } },
  },
];
