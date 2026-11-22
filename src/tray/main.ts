// A separate entry point, loaded by `tray.html` (task M3-57): the tray window gets its own
// bundle and capability set (`src-tauri/capabilities/tray.json`, ADR 0010 point 4), kept apart
// from `src/main.ts`'s main-window entry -- the router, every feature store the main window
// loads, and the main window's own commands never ship in the tray's own chunk at all.
//
// M3-58 replaced the placeholder with the real view. It needs the app's stylesheet (the main entry
// imports the same one) and does its own theme loading, since this window shares no state with the main one.
import '../styles/main.css';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import TrayView from '@/features/tray/TrayView.vue';

createApp(TrayView).use(createPinia()).mount('#app');
