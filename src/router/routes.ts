// Route table (task M2-06). ipc-contract.md's `ConfigKind` only covers four slug-addressable
// kinds (command/skill/outputStyle/agent) -- hooks/mcp/plugins/models each have their own
// dedicated, differently-shaped commands (listHooks/listMcpServers/listPlugins/listModels), so
// they are not reachable through the generic /config/:kind route at all; each gets its own
// static path instead.

import type { RouteRecordRaw } from 'vue-router';
import ConfigDetailRoute from './ConfigDetailRoute.vue';
import ConfigListRoute from './ConfigListRoute.vue';

export const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/sessions' },
  // AC's own pattern is `/sessions/:projectId/:sessionId?` (projectId required); the bare
  // `/sessions` (no project picked yet) is the same component with both params absent.
  { path: '/sessions', name: 'sessions', component: () => import('@/features/session/SessionsPage.vue') },
  {
    path: '/sessions/:projectId/:sessionId?',
    name: 'session-detail',
    component: () => import('@/features/session/SessionsPage.vue'),
  },
  { path: '/todos', name: 'todos', component: () => import('@/features/todos/TodosPage.vue') },
  { path: '/history', name: 'history', component: () => import('@/features/history/HistoryPage.vue') },
  { path: '/plans', name: 'plans', component: () => import('@/features/plans/PlansPage.vue') },
  { path: '/memory', name: 'memory', component: () => import('@/features/memory/MemoryPage.vue') },
  { path: '/analytics', name: 'analytics', component: () => import('@/features/analytics/AnalyticsPage.vue') },
  { path: '/settings', name: 'settings', component: () => import('@/features/settings/SettingsPage.vue') },
  { path: '/help', name: 'help', component: () => import('@/features/help/HelpPage.vue') },
  { path: '/about', name: 'about', component: () => import('@/features/about/AboutDialog.vue') },

  // The generic pair the AC names, for the four real ConfigKind values.
  { path: '/config/:kind', name: 'config-list', component: ConfigListRoute },
  { path: '/config/:kind/:scope/:slug', name: 'config-detail', component: ConfigDetailRoute },

  // The other four config screens: not ConfigKind-shaped, so not under the generic route.
  { path: '/config/hooks', name: 'hooks', component: () => import('@/features/config/hooks/HooksPage.vue') },
  { path: '/config/mcp', name: 'mcp', component: () => import('@/features/config/mcp/McpServersPage.vue') },
  {
    path: '/config/plugins',
    name: 'plugins',
    component: () => import('@/features/config/plugins/PluginsPage.vue'),
  },
  { path: '/config/models', name: 'models', component: () => import('@/features/config/models/ModelsPage.vue') },
];
