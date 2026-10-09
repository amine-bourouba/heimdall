// @vitest-environment node
// This spec reads a file and does arithmetic, nothing DOM-shaped — happy-dom (the project
// default) externalizes Node built-ins like node:fs as browser-incompatible, so this one file
// opts back into the plain Node environment instead.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { themes } from './themes';

// Reads the real, committed tokens.css rather than re-encoding palette data a second time here —
// the same anti-drift reasoning as M2-01's DTO fixtures: a second hand-copied source of the same
// values is exactly how a contrast test ends up checking colours nobody ships.
const tokensCssPath = fileURLToPath(new URL('../styles/tokens.css', import.meta.url));
const tokensCss = readFileSync(tokensCssPath, 'utf-8');

type Mode = 'base' | 'dark';

/** The four tokens this spec checks are always set with full opacity in every theme (verified
 * against PoirotTheme.swift: only border/diff tokens ever carry an opacity), so a plain #RRGGBB
 * capture is enough — no rgba() parsing needed. */
const TOKEN_PATTERN = /--(bg-app|bg-card|text-primary|text-secondary):\s*#([0-9A-Fa-f]{6});/g;

function extractTokens(block: string): Record<string, string> {
  const found: Record<string, string> = {};
  for (const match of block.matchAll(TOKEN_PATTERN)) {
    found[match[1]] = match[2];
  }
  return found;
}

function themeColors(name: string, mode: Mode): Record<string, string> {
  const baseMatch = tokensCss.match(new RegExp(`\\[data-theme="${name}"\\] \\{([^}]*)\\}`));
  if (!baseMatch) throw new Error(`tokens.css has no [data-theme="${name}"] block`);
  if (mode === 'base') return extractTokens(baseMatch[1]);

  const darkMediaMatch = tokensCss.match(
    new RegExp(`@media \\(prefers-color-scheme: dark\\) \\{\\s*\\[data-theme="${name}"\\] \\{([^}]*)\\}`),
  );
  if (!darkMediaMatch) throw new Error(`tokens.css has no dark-mode override for [data-theme="${name}"]`);
  return extractTokens(darkMediaMatch[1]);
}

function srgbToLinear(channel: number): number {
  const c = channel / 255;
  return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

function relativeLuminance(hex: string): number {
  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);
  return 0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b);
}

/** WCAG 2's own contrast-ratio formula (relative luminance, lighter over darker, +0.05 each). */
function contrastRatio(hexA: string, hexB: string): number {
  const la = relativeLuminance(hexA);
  const lb = relativeLuminance(hexB);
  const [lighter, darker] = la >= lb ? [la, lb] : [lb, la];
  return (lighter + 0.05) / (darker + 0.05);
}

const AA_NORMAL_TEXT = 4.5;

describe('every theme meets WCAG AA (4.5:1) for primary/secondary text on bg-app/bg-card', () => {
  for (const theme of themes) {
    const modes: Mode[] = theme.isDark === 'adaptive' ? ['base', 'dark'] : ['base'];
    for (const mode of modes) {
      const label = theme.isDark === 'adaptive' ? `${theme.id} (${mode === 'base' ? 'light' : 'dark'})` : theme.id;

      it(`${label}: text-primary on bg-app and bg-card`, () => {
        const c = themeColors(theme.id, mode);
        expect(contrastRatio(c['text-primary'], c['bg-app'])).toBeGreaterThanOrEqual(AA_NORMAL_TEXT);
        expect(contrastRatio(c['text-primary'], c['bg-card'])).toBeGreaterThanOrEqual(AA_NORMAL_TEXT);
      });

      it(`${label}: text-secondary on bg-app and bg-card`, () => {
        const c = themeColors(theme.id, mode);
        expect(contrastRatio(c['text-secondary'], c['bg-app'])).toBeGreaterThanOrEqual(AA_NORMAL_TEXT);
        expect(contrastRatio(c['text-secondary'], c['bg-card'])).toBeGreaterThanOrEqual(AA_NORMAL_TEXT);
      });
    }
  }
});
