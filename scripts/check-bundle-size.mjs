// Fails when any chunk exceeds its gzip ceiling (port-plan.md §9: "initial route ≤ 250 KB gzip;
// each lazy route ≤ 150 KB gzip; ECharts only in the analytics chunk", M2-28). Reads
// `dist/.vite/manifest.json` (`vite.config.ts`'s own `build.manifest: true`, added by this task)
// rather than guessing which output file is "the initial bundle" vs "a lazy route" from filenames
// alone -- Shiki lazily imports well over a hundred per-language grammar chunks of its own
// (confirmed directly in a real build: `emacs-lisp-*.js` alone is 200 KB gzip), and none of those
// are a *route* at all, so a naive "every non-entry .js file in dist/assets" rule would wrongly
// fail on them.
//
// Run after `npm run build` (not run by this script): `node scripts/check-bundle-size.mjs`.
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

// Vite's own build output (confirmed directly: `npm run build`'s own console log) reports gzip
// size in decimal kB (bytes / 1000), not 1024-based KiB -- matching that convention keeps this
// script's PASS/FAIL agreeing with the numbers the build step itself already printed.
const KB = 1000;
export const INITIAL_ROUTE_BUDGET_BYTES = 250 * KB;
export const LAZY_ROUTE_BUDGET_BYTES = 150 * KB;

function gzipSize(filePath) {
  return gzipSync(readFileSync(filePath)).length;
}

/** @param {unknown} manifest parsed `dist/.vite/manifest.json` @param {string} distDir */
export function checkBundleSize(manifest, distDir) {
  const results = [];

  const entryKey = Object.keys(manifest).find((k) => manifest[k].isEntry);
  if (!entryKey) throw new Error('manifest.json has no isEntry chunk -- was it built with build.manifest: true?');
  const entry = manifest[entryKey];

  const initialBytes = gzipSize(path.join(distDir, entry.file));
  results.push({ name: `initial route (${entry.file})`, bytes: initialBytes, budget: INITIAL_ROUTE_BUDGET_BYTES });

  for (const routeModule of entry.dynamicImports ?? []) {
    const route = manifest[routeModule];
    if (!route?.file) continue;
    const bytes = gzipSize(path.join(distDir, route.file));
    results.push({ name: `lazy route ${routeModule} (${route.file})`, bytes, budget: LAZY_ROUTE_BUDGET_BYTES });
  }

  return results;
}

/** ECharts is budgeted ("ECharts only in the analytics chunk") by *who imports it*, not by
 * inspecting minified bundle contents, which gives no reliable per-module attribution once
 * Rollup inlines a dependency into its importer's chunk -- a source-level grep is both simpler
 * and the thing actually being guaranteed: nothing outside the analytics feature ever pulls
 * ECharts' considerable weight into its own route. @param {string} srcDir */
export function checkEchartsPlacement(srcDir) {
  const violations = [];
  const allowedDir = path.join(srcDir, 'features', 'analytics');

  function walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(full);
      } else if (/\.(ts|vue)$/.test(entry.name)) {
        const text = readFileSync(full, 'utf-8');
        if (/from\s+['"](vue-)?echarts/.test(text) && !full.startsWith(allowedDir)) {
          violations.push(`${path.relative(srcDir, full)} imports echarts outside src/features/analytics/`);
        }
      }
    }
  }
  walk(srcDir);
  return violations;
}

function formatKb(bytes) {
  return (bytes / KB).toFixed(2);
}

function main() {
  const distDir = path.join(ROOT, 'dist');
  const manifestPath = path.join(distDir, '.vite', 'manifest.json');
  if (!existsSync(manifestPath)) {
    console.error(`No manifest at ${manifestPath} -- run \`npm run build\` first.`);
    process.exitCode = 1;
    return;
  }

  const manifest = JSON.parse(readFileSync(manifestPath, 'utf-8'));
  const results = checkBundleSize(manifest, distDir);
  const echartsViolations = checkEchartsPlacement(path.join(ROOT, 'src'));

  let failed = false;
  for (const r of results) {
    const ok = r.bytes <= r.budget;
    if (!ok) failed = true;
    console.log(`${ok ? 'PASS' : 'FAIL'}  ${r.name}: ${formatKb(r.bytes)} KB gzip (budget ${formatKb(r.budget)} KB)`);
  }
  for (const v of echartsViolations) {
    failed = true;
    console.log(`FAIL  ${v}`);
  }
  if (echartsViolations.length === 0) console.log('PASS  ECharts is imported only from src/features/analytics/ (or not at all yet)');

  process.exitCode = failed ? 1 : 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
