// Wraps prefs.ts's colorTheme/fontScale (ADR 0007: both live in the one Rust-owned Prefs struct,
// not a store of their own) with the metadata catalog (src/theme/themes.ts, M2-02) and the DOM
// side effects tokens.css/main.css actually need: `data-theme` on <html> and `--font-scale`.

import { computed } from 'vue';
import { defineStore } from 'pinia';
import { themes } from '@/theme/themes';
import type { ThemeMeta } from '@/theme/themes';
import { usePrefsStore } from './prefs';
import type { ColorThemeId } from '@/ipc/bindings';

const FONT_SCALE_MIN = 0.75;
const FONT_SCALE_MAX = 1.5;
const FONT_SCALE_STEP = 0.05;

/** Avoids classic 0.1 + 0.2 float drift across repeated 0.05 steps (src-tauri/src/prefs.rs hit
 * the same class of bug in the Rust validator this value must stay compatible with). */
function round2(n: number): number {
  return Math.round(n * 100) / 100;
}

export const useThemeStore = defineStore('theme', () => {
  const prefsStore = usePrefsStore();

  const current = computed<ThemeMeta>(
    () => themes.find((t) => t.id === prefsStore.colorTheme) ?? themes[0],
  );
  const isDark = computed(() => current.value.isDark);
  const fontScale = computed(() => prefsStore.fontScale);

  function applyToDocument() {
    document.documentElement.dataset.theme = current.value.id;
    document.documentElement.style.setProperty('--font-scale', String(fontScale.value));
  }

  async function setColorTheme(id: ColorThemeId) {
    await prefsStore.update({ colorTheme: id });
    applyToDocument();
  }

  async function increaseFontScale() {
    await prefsStore.update({ fontScale: round2(Math.min(fontScale.value + FONT_SCALE_STEP, FONT_SCALE_MAX)) });
    applyToDocument();
  }

  async function decreaseFontScale() {
    await prefsStore.update({ fontScale: round2(Math.max(fontScale.value - FONT_SCALE_STEP, FONT_SCALE_MIN)) });
    applyToDocument();
  }

  async function resetFontScale() {
    await prefsStore.update({ fontScale: 1.0 });
    applyToDocument();
  }

  return { themes, current, isDark, fontScale, applyToDocument, setColorTheme, increaseFontScale, decreaseFontScale, resetFontScale };
});
