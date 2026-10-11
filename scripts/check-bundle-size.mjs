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

/** Every chunk reachable from `key` by eager (non-dynamic) `imports`, `key` itself included --
 * used to find everything the initial-route entry already loads, so a lazy route's own weight
 * can exclude it (see `chunkWeightBytes`). */
function collectEagerClosure(manifest, key, seen = new Set()) {
  if (seen.has(key)) return seen;
  seen.add(key);
  for (const importedKey of manifest[key]?.imports ?? []) collectEagerClosure(manifest, importedKey, seen);
  return seen;
}

/** A chunk's real *additional* weight is its own file plus every chunk it eagerly `imports`
 * (recursively) that isn't already in `alreadyLoaded`. Two reasons this matters, both found
 * empirically while actually running this against M3-57's new multi-page build, not assumed:
 * (1) a second entry point (`tray.html`) makes Rollup factor out a shared chunk (e.g. Pinia) that
 * both entries `import` instead of each bundling their own copy -- measuring `entry.file` alone
 * would silently under-count exactly the part that got split out, for the *initial* route. (2)
 * every lazy route chunk's own `imports` list includes the main entry itself (`index.html`) --
 * Rollup's own bookkeeping for "this dynamic chunk assumes the entry's module registry is already
 * loaded," true and free at runtime since the entry always loads first -- so summing a lazy
 * route's imports *without* excluding what the entry already loaded would double-count the whole
 * entry bundle into every single route's own budget (confirmed: before this exclusion, every
 * stub page jumped to ~55 KB and `SessionsPage.vue` read as a false 175 KB "failure", for bytes
 * that were never additional downloads). Dynamic (`import()`-lazy) imports are never followed
 * into this sum at all -- those are each their own separate, separately-budgeted chunk. */
function chunkWeightBytes(manifest, distDir, key, alreadyLoaded, seen = new Set()) {
  if (seen.has(key) || alreadyLoaded.has(key)) return 0;
  seen.add(key);
  const chunk = manifest[key];
  if (!chunk?.file) return 0;
  let bytes = gzipSize(path.join(distDir, chunk.file));
  for (const importedKey of chunk.imports ?? []) bytes += chunkWeightBytes(manifest, distDir, importedKey, alreadyLoaded, seen);
  return bytes;
}

/** @param {unknown} manifest parsed `dist/.vite/manifest.json` @param {string} distDir */
export function checkBundleSize(manifest, distDir) {
  const results = [];

  // Named explicitly, not `Object.keys(manifest).find(isEntry)`: M3-57's multi-page build
  // (`tray.html`) means the manifest now has *two* `isEntry` chunks, and the budget this
  // function checks ("initial route") is specifically the main window's own bundle, not
  // whichever entry happens to iterate first.
  const entry = manifest['index.html'];
  if (!entry) throw new Error('manifest.json has no "index.html" entry -- was it built with build.manifest: true?');

  const initialBytes = chunkWeightBytes(manifest, distDir, 'index.html', new Set());
  results.push({ name: `initial route (${entry.file})`, bytes: initialBytes, budget: INITIAL_ROUTE_BUDGET_BYTES });

  // Everything the initial route already loads -- excluded from each lazy route's own weight
  // below, since it's already paid for by the time any route lazy-loads (see
  // `chunkWeightBytes`'s own comment for why this matters, confirmed empirically).
  const alreadyLoaded = collectEagerClosure(manifest, 'index.html');

  for (const routeModule of entry.dynamicImports ?? []) {
    const route = manifest[routeModule];
    if (!route?.file) continue;
    const bytes = chunkWeightBytes(manifest, distDir, routeModule, alreadyLoaded);
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
