// Hash history (port-plan.md decision P-3): no server to configure for deep links in a webview,
// and it gives Vue Router's own native back/forward semantics for free -- see
// src/composables/useNavigation.ts for how canGoBack/canGoForward and cache hydration lean on
// that directly instead of reimplementing AppState.swift's own hand-rolled history stack.
import { createRouter, createWebHashHistory } from 'vue-router';
import { routes } from './routes';

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
