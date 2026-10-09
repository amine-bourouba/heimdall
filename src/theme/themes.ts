// The 20 themes' identity and metadata (PoirotTheme.swift's ColorTheme enum): `id` is the
// `data-theme` attribute value tokens.css selects on, `label` is ColorTheme.label verbatim, and
// `isDark` is ColorTheme.isDark (PoirotTheme.swift:283-289) — 'adaptive' for the three themes
// that follow `prefers-color-scheme` (default/solarized/highContrast), a fixed boolean for the
// other 17, ported from a7t/chat, which are single-appearance. Colour values themselves live only
// in tokens.css, never duplicated here — this catalog drives the theme-picker UI and
// `color-scheme` only.

export interface ThemeMeta {
  id: string;
  label: string;
  isDark: boolean | 'adaptive';
}

export const themes: ThemeMeta[] = [
  { id: 'default', label: 'Default', isDark: 'adaptive' },
  { id: 'solarized', label: 'Solarized', isDark: 'adaptive' },
  { id: 'highContrast', label: 'High Contrast', isDark: 'adaptive' },
  { id: 'charcoal', label: 'Charcoal', isDark: true },
  { id: 'tealNight', label: 'Teal Night', isDark: true },
  { id: 'white', label: 'White', isDark: false },
  { id: 'pastel', label: 'Pastel', isDark: false },
  { id: 'greenTheme', label: 'Green', isDark: false },
  { id: 'nightshade', label: 'Nightshade', isDark: true },
  { id: 'fjord', label: 'Fjord', isDark: true },
  { id: 'fjordLight', label: 'Fjord Light', isDark: false },
  { id: 'harbor', label: 'Harbor', isDark: true },
  { id: 'mocha', label: 'Mocha', isDark: true },
  { id: 'rosewood', label: 'Rosewood', isDark: true },
  { id: 'ember', label: 'Ember', isDark: true },
  { id: 'parchment', label: 'Parchment', isDark: false },
  { id: 'deepSea', label: 'Deep Sea', isDark: true },
  { id: 'daybreak', label: 'Daybreak', isDark: false },
  { id: 'citrus', label: 'Citrus', isDark: true },
  { id: 'evergreen', label: 'Evergreen', isDark: true },
];
