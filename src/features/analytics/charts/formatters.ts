// Ported from AnalyticsFormatters.swift (M3-33; docs/spec/analytics.md). No Swift XCTest file
// exists for these -- expected values below are derived directly from the Swift formulas
// themselves, the same way M3-31 did for parseStatsDate/friendlyModelName.
//
// `parseDate`/`dateToString`/`formatFirstSessionDate` already live in `../store.ts`
// (`parseStatsDate`) and `AnalyticsDashboard.vue` (`formatFirstSessionDate`) respectively --
// only the formulas every chart component needs are duplicated here, one lazy-loaded module away
// from the eager chunk those two call sites stay in.
import { format } from 'echarts/core';

export function formatLargeNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

export function formatCost(n: number): string {
  if (n >= 1000) return `$${n.toFixed(0)}`;
  if (n >= 100) return `$${n.toFixed(1)}`;
  return `$${n.toFixed(2)}`;
}

/** `AnalyticsFormatters.formatHour` (hour:0-23 -> "12AM"/"1PM"/...). */
export function formatHour(hour: number): string {
  const period = hour >= 12 ? 'PM' : 'AM';
  const displayHour = hour === 0 ? 12 : hour > 12 ? hour - 12 : hour;
  return `${displayHour}${period}`;
}

/** `store.ts`'s own `parseStatsDate` builds every `Date` from LOCAL y/m/d components specifically
 * to avoid UTC-midnight parsing shifting day boundaries -- converting one back into a
 * `yyyy-MM-dd` key has the identical pitfall in the other direction: `.toISOString()` reads the
 * UTC wall-clock, silently shifting the key back a day for any positive UTC offset. Local
 * getters only (a real bug, already found and fixed once in this exact code, M3-32). */
export function localDateKey(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

/** ADR 0010 control 1 / M3-33 AC2: every tooltip formatter that embeds a data-derived string
 * (a model name, ultimately a raw JSONL `model` field when `friendlyModelName()` doesn't
 * recognise it -- attacker-influenceable, same threat model as every other disk-read string this
 * app renders) must pass it through this before it reaches an ECharts tooltip, which is real
 * HTML/`innerHTML`, not a Vue template -- Vue's own auto-escaping does not apply to it. */
export function escapeForTooltip(value: string): string {
  return format.encodeHTML(value);
}
