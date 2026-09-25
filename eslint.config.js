import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';

export default defineConfig(
  {
    ignores: ['dist/', 'node_modules/', 'src-tauri/', 'target/', 'coverage/', 'fixtures/', 'playwright-report/',
      'test-results/', 'src/ipc/bindings.ts', 'docs/'],
  },
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  { files: ['**/*.vue'], languageOptions: { parserOptions: { parser: tseslint.parser } } },
  {
    rules: {
      // ADR 0010: renderMarkdown() in MarkdownView.vue is the only path into v-html.
      'vue/no-v-html': 'error',
      // ADR 0002/0004: IPC only through src/ipc/client.ts over the generated bindings.
      'no-restricted-imports': ['error', {
        paths: [{ name: '@tauri-apps/api/core', message: 'Use src/ipc/client.ts (generated bindings) instead.' }],
      }],
    },
  },
  { files: ['src/lib/markdown/MarkdownView.vue'], rules: { 'vue/no-v-html': 'off' } },
  { files: ['src/ipc/**'], rules: { 'no-restricted-imports': 'off' } },
);
