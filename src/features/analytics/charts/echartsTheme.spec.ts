// M3-33 AC3: "Charts re-theme when the theme store changes." `useEchartsTheme()`'s computeds
// read live CSS custom properties but only re-run when `themeStore.current.id` changes (the
// explicit `void themeStore.current.id` reactive dependency in each one) -- this proves that
// wiring actually works, not just that it doesn't throw. No component/VTU needed: Pinia stores
// and `computed()` both work outside a component's `setup()` as long as Pinia is active.
import { createPinia, setActivePinia } from 'pinia';
import { describe, expect, it } from 'vitest';
import { usePrefsStore } from '@/stores/prefs';
import type { Prefs } from '@/ipc/bindings';
import { colorForModel, NEUTRAL_FALLBACK, useEchartsTheme } from './echartsTheme';

function makePrefs(overrides: Partial<Prefs> = {}): Prefs {
  return {
    schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1.0, sessionLayout: 'grid', configProjectPath: null, configLayouts: {},
    showAgentSessions: false, textEditor: 'vscode', preferredTerminal: 'terminal', openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true,
    autoExpandBlocks: true, parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [],
    ...overrides,
  };
}

describe('useEchartsTheme', () => {
  it('re-reads CSS custom properties only when the active theme id changes', () => {
    setActivePinia(createPinia());
    const prefsStore = usePrefsStore();
    prefsStore.prefs = makePrefs({ colorTheme: 'default' });

    document.documentElement.style.setProperty('--accent', '#111111');
    const { accentColor } = useEchartsTheme();
    expect(accentColor.value).toBe('#111111');

    // Changing the CSS value alone, with no theme change, must NOT be picked up (that's the
    // whole point of the reactive dependency being `themeStore.current.id`, not a DOM watcher).
    document.documentElement.style.setProperty('--accent', '#222222');
    expect(accentColor.value).toBe('#111111');

    // Changing the theme re-triggers the computed, which re-reads the (now-different) CSS value.
    prefsStore.prefs = makePrefs({ colorTheme: 'charcoal' });
    expect(accentColor.value).toBe('#222222');
  });

  it('exposes all 6 palette slots plus the text/border/card colours', () => {
    setActivePinia(createPinia());
    usePrefsStore().prefs = makePrefs();
    const theme = useEchartsTheme();
    expect(theme.palette.value).toHaveLength(6);
    expect(theme).toHaveProperty('textSecondary');
    expect(theme).toHaveProperty('textTertiary');
    expect(theme).toHaveProperty('borderColor');
    expect(theme).toHaveProperty('bgCardColor');
  });
});

describe('colorForModel (analytics.md §12.3: one stable colour per model, shared across charts)', () => {
  const palette = ['#a', '#b', '#c'];

  it('assigns the Nth palette slot to the Nth sorted model name', () => {
    expect(colorForModel('m2', ['m1', 'm2', 'm3'], palette)).toBe('#b');
  });

  it('falls back to the neutral colour for a 7th+ model past the palette length (no wraparound)', () => {
    expect(colorForModel('m3', ['m1', 'm2', 'm3'], ['#a', '#b'])).toBe(NEUTRAL_FALLBACK);
  });

  it('falls back to the neutral colour for a model missing from the known set entirely', () => {
    expect(colorForModel('unknown-model', ['m1'], palette)).toBe(NEUTRAL_FALLBACK);
  });
});
