// A separate entry point, loaded by `tray.html` (task M3-57): the tray window gets its own
// bundle and capability set (`src-tauri/capabilities/tray.json`, ADR 0010 point 4), kept apart
// from `src/main.ts`'s main-window entry -- the router, every feature store the main window
// loads, and the main window's own commands never ship in the tray's own chunk at all.
//
// Placeholder content only, same M2-06-era stub rule every other not-yet-built screen follows
// (`src/features/*/index.ts`'s own `available: false` convention) -- M3-58 replaces this with
// the real tray view and Menu Bar settings tab. Pinia is wired up already (not deferred to
// M3-58) since the tray capability already grants `allow-get-prefs`, and the real view will need
// the same prefs/theme stores `src/main.ts` sets up for the main window.
import { createApp, h } from 'vue';
import { createPinia } from 'pinia';

const TrayApp = { render: () => h('div', 'Tray is not built yet.') };

createApp(TrayApp).use(createPinia()).mount('#app');
