import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

// Reuses vite.config.ts's own plugins/alias (mergeConfig, the documented Vitest pattern) rather
// than redeclaring them here, so the two configs can never silently drift apart.
export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'happy-dom',
      // Vitest's own default glob would also pick up scripts/scrub-claude-home.test.mjs, which
      // is a node:test file run via `node --test` (docs/progress-log.md, M0-13), not a Vitest spec.
      // `tests/` (tsconfig.json already anticipates it) holds specs that aren't colocated with one
      // component/module, starting with the cross-cutting security corpora (M2-25, M3-35).
      include: ['src/**/*.{test,spec}.ts', 'tests/**/*.{test,spec}.ts'],
      setupFiles: ['./src/test/setup.ts'],
      coverage: {
        provider: 'v8',
        // port-plan.md §8: 80% lines on stores/composables/feature stores once they exist
        // (M2-05+); an empty glob today is not a failure, just nothing yet to enforce.
        include: ['src/stores/**', 'src/composables/**', 'src/features/**/store.ts'],
        thresholds: { lines: 80 },
      },
    },
  }),
);
