import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { Prefs } from '@/ipc/bindings';
import { usePrefsStore } from './prefs';
import { useThemeStore } from './theme';

function prefsWith(overrides: Partial<Prefs> = {}): Prefs {
  return {
    schemaVersion: 1,
    showMenuBarIcon: true,
    hasCompletedOnboarding: false,
    colorTheme: 'default',
    fontScale: 1.0,
    sessionLayout: 'grid',
    configProjectPath: null,
    configLayouts: {},
    showAgentSessions: false,
    textEditor: 'vscode',
    preferredTerminal: 'terminal',
    openTerminalOnBash: false,
    claudeCodePath: null,
    showAnimations: true,
    wrapCodeLines: true,
    autoExpandBlocks: true,
    parseMarkdownInResults: true,
    globalSearchMode: 'fuzzy',
    customMemoryFolders: [],
    ...overrides,
  };
}

/** Mocks get_prefs with `prefs` and update_prefs as a real merge over it (mirrors what
 * src-tauri/src/prefs.rs's own update() actually does), so a store under test gets back
 * whatever the real backend would compute, not a value this test pre-decided. */
function mockPrefsBackend(prefs: Prefs) {
  let current = prefs;
  mockIPC((cmd, args) => {
    if (cmd === 'get_prefs') return current;
    if (cmd === 'update_prefs') {
      const patch = (args as { patch: Record<string, unknown> }).patch;
      current = { ...current, ...Object.fromEntries(Object.entries(patch).filter(([, v]) => v !== null)) };
      return current;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  return () => current;
}

beforeEach(() => {
  setActivePinia(createPinia());
  document.documentElement.removeAttribute('data-theme');
  document.documentElement.style.removeProperty('--font-scale');
});

// Ported from PoirotTests/App/ColorThemeTests.swift.
describe('useThemeStore: catalog (ColorThemeTests)', () => {
  it('has all 20 themes, originals leading, ported themes present', () => {
    const store = useThemeStore();
    expect(store.themes).toHaveLength(20);
    expect(store.themes.slice(0, 3).map((t) => t.id)).toEqual(['default', 'solarized', 'highContrast']);
    expect(store.themes.map((t) => t.id)).toContain('charcoal');
    expect(store.themes.map((t) => t.id)).toContain('nightshade');
    expect(store.themes.map((t) => t.id)).toContain('evergreen');
  });

  it('marks fixed themes dark/light and adaptive originals "adaptive"', () => {
    const store = useThemeStore();
    const byId = (id: string) => store.themes.find((t) => t.id === id)!;
    expect(byId('default').isDark).toBe('adaptive');
    expect(byId('solarized').isDark).toBe('adaptive');
    expect(byId('charcoal').isDark).toBe(true);
    expect(byId('nightshade').isDark).toBe(true);
    expect(byId('white').isDark).toBe(false);
    expect(byId('pastel').isDark).toBe(false);
  });

  it('labels match PoirotTheme.swift', () => {
    const store = useThemeStore();
    const byId = (id: string) => store.themes.find((t) => t.id === id)!;
    expect(byId('default').label).toBe('Default');
    expect(byId('solarized').label).toBe('Solarized');
    expect(byId('highContrast').label).toBe('High Contrast');
  });

  it("current/isDark reflect the active prefs.colorTheme, defaulting to 'default'", async () => {
    mockPrefsBackend(prefsWith({ colorTheme: 'nightshade' }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    expect(themeStore.current.id).toBe('nightshade');
    expect(themeStore.isDark).toBe(true);
  });

  it('setColorTheme persists through update_prefs and updates <html data-theme>', async () => {
    mockPrefsBackend(prefsWith());
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    await themeStore.setColorTheme('solarized');

    expect(themeStore.current.id).toBe('solarized');
    expect(document.documentElement.dataset.theme).toBe('solarized');
  });
});

// Ported from PoirotTests/App/AppStateTests.swift's font-scale cases (increaseFontScale/
// decreaseFontScale/resetFontScale) -- the rest of that file (project filtering, navigation
// history, session cache) belongs to later M2 tasks (sessions-store, sessions-navigation), not
// this one's own Outputs.
describe('useThemeStore: font scale (AppStateTests font-scale cases)', () => {
  it('increaseFontScale increments by 0.05', async () => {
    mockPrefsBackend(prefsWith({ fontScale: 1.0 }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    await themeStore.increaseFontScale();
    expect(themeStore.fontScale).toBe(1.05);
  });

  it('increaseFontScale clamps at 1.5', async () => {
    mockPrefsBackend(prefsWith({ fontScale: 1.5 }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    await themeStore.increaseFontScale();
    expect(themeStore.fontScale).toBe(1.5);
  });

  it('decreaseFontScale decrements by 0.05', async () => {
    mockPrefsBackend(prefsWith({ fontScale: 1.0 }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    await themeStore.decreaseFontScale();
    expect(themeStore.fontScale).toBe(0.95);
  });

  it('decreaseFontScale clamps at 0.75', async () => {
    mockPrefsBackend(prefsWith({ fontScale: 0.75 }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    await themeStore.decreaseFontScale();
    expect(themeStore.fontScale).toBe(0.75);
  });

  it('resetFontScale sets 1.0', async () => {
    mockPrefsBackend(prefsWith({ fontScale: 1.3 }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    await themeStore.resetFontScale();
    expect(themeStore.fontScale).toBe(1.0);
  });

  it('applyToDocument sets --font-scale on <html> from the current value', async () => {
    mockPrefsBackend(prefsWith({ fontScale: 1.2 }));
    const prefsStore = usePrefsStore();
    const themeStore = useThemeStore();
    await prefsStore.load();

    themeStore.applyToDocument();
    expect(document.documentElement.style.getPropertyValue('--font-scale')).toBe('1.2');
  });
});
