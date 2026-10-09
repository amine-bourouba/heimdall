// Owns the one Rust-owned Prefs struct (ADR 0007): loads it via get_prefs, and writes any subset
// of fields back via update_prefs, which the backend validates and is the single source of truth
// (the store takes whatever it returns, not an optimistic local guess).

import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc } from '@/ipc/client';
import type { Prefs, PrefsPatch } from '@/ipc/bindings';

/** `PrefsPatch`'s fields are all required-but-nullable ("null" means "leave unchanged") -- this
 * is the "change nothing" patch a partial update is layered onto. */
function blankPatch(): PrefsPatch {
  return {
    showMenuBarIcon: null,
    hasCompletedOnboarding: null,
    colorTheme: null,
    fontScale: null,
    sessionLayout: null,
    configLayouts: null,
    showAgentSessions: null,
    textEditor: null,
    preferredTerminal: null,
    openTerminalOnBash: null,
    claudeCodePath: null,
    showAnimations: null,
    wrapCodeLines: null,
    autoExpandBlocks: null,
    parseMarkdownInResults: null,
    globalSearchMode: null,
  };
}

export const usePrefsStore = defineStore('prefs', () => {
  const prefs = ref<Prefs | null>(null);

  const fontScale = computed(() => prefs.value?.fontScale ?? 1.0);
  const colorTheme = computed(() => prefs.value?.colorTheme ?? 'default');
  const showAnimations = computed(() => prefs.value?.showAnimations ?? true);

  async function load() {
    prefs.value = await ipc.getPrefs();
  }

  /** Validates and persists only the given fields; every other field is left unchanged. */
  async function update(patch: Partial<PrefsPatch>) {
    prefs.value = await ipc.updatePrefs({ ...blankPatch(), ...patch });
  }

  return { prefs, fontScale, colorTheme, showAnimations, load, update };
});
