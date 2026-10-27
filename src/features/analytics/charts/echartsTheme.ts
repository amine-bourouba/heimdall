// Theme-aware palette shared by every chart (M3-33; analytics.md §5: 6 fixed slots, no
// wraparound). `useEchartsTheme()` is a composable, not a plain export, because each of these is
// a `computed()` reading live CSS custom properties -- every chart component that calls it needs
// its OWN computed instances so Vue tracks each one's own re-render, not one shared instance that
// could go stale for components that mounted after the last theme change.
import { computed } from 'vue';
import { useThemeStore } from '@/stores/theme';

const PALETTE_VARS = ['--accent', '--color-blue', '--color-purple', '--color-teal', '--color-green', '--color-orange'] as const;
export const NEUTRAL_FALLBACK = '#8E8E93';

function resolveVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

export function useEchartsTheme() {
  const themeStore = useThemeStore();
  const palette = computed<string[]>(() => {
    void themeStore.current.id; // reactive dependency: re-read the CSS custom properties whenever the active theme changes
    return PALETTE_VARS.map(resolveVar);
  });
  const textSecondary = computed(() => {
    void themeStore.current.id;
    return resolveVar('--text-secondary');
  });
  const textTertiary = computed(() => {
    void themeStore.current.id;
    return resolveVar('--text-tertiary');
  });
  const borderColor = computed(() => {
    void themeStore.current.id;
    return resolveVar('--border');
  });
  const accentColor = computed(() => {
    void themeStore.current.id;
    return resolveVar('--accent');
  });
  const bgCardColor = computed(() => {
    void themeStore.current.id;
    return resolveVar('--bg-card');
  });
  return { palette, textSecondary, textTertiary, borderColor, accentColor, bgCardColor };
}

/** analytics.md §12.3's own "color follows the entity, never its rank" fix (M3-32): one stable
 * colour per model name, looked up against `allModelNames` (computed once in `store.ts`, shared
 * by every chart, so the same model never renders a different colour on different charts). A
 * 7th+ model (beyond the fixed 6-colour palette, matching Swift's own no-wraparound cap) gets
 * `NEUTRAL_FALLBACK` rather than silently reusing an earlier colour. */
export function colorForModel(model: string, allModelNames: string[], palette: string[]): string {
  const idx = allModelNames.indexOf(model);
  return idx >= 0 && idx < palette.length ? palette[idx] : NEUTRAL_FALLBACK;
}
