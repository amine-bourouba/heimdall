import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

// Worktree N runs on port 1420 + N (AI_INSTRUCTIONS.md, "Parallel development").
const port = Number(process.env.HEIMDALL_DEV_PORT ?? 1420);
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
  clearScreen: false,
  server: {
    port,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: port + 1 } : undefined,
    watch: { ignored: ['**/src-tauri/**', '**/target/**', '**/fixtures/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // WebView2 is Chromium-based; WKWebView and WebKitGTK are Safari-class engines.
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome110' : 'safari16',
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
});
