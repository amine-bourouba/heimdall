import { describe, expect, it } from 'vitest';
import { escapeForTooltip, formatCost, formatHour, formatLargeNumber, localDateKey } from './formatters';

describe('formatLargeNumber (AnalyticsFormatters.formatLargeNumber)', () => {
  it.each([
    [0, '0'],
    [999, '999'],
    [1000, '1.0K'],
    [1500, '1.5K'],
    [999_999, '1000.0K'],
    [1_000_000, '1.0M'],
    [2_500_000, '2.5M'],
  ])('formats %i as %s', (n, expected) => {
    expect(formatLargeNumber(n)).toBe(expected);
  });
});

describe('formatCost (AnalyticsFormatters.formatCost)', () => {
  it.each([
    [0, '$0.00'],
    [0.5, '$0.50'],
    [99.99, '$99.99'],
    [100, '$100.0'],
    [999.99, '$1000.0'],
    [1000, '$1000'],
    [12_345, '$12345'],
  ])('formats %f as %s', (n, expected) => {
    expect(formatCost(n)).toBe(expected);
  });
});

describe('formatHour (AnalyticsFormatters.formatHour)', () => {
  it.each([
    [0, '12AM'],
    [1, '1AM'],
    [11, '11AM'],
    [12, '12PM'],
    [13, '1PM'],
    [23, '11PM'],
  ])('formats hour %i as %s', (hour, expected) => {
    expect(formatHour(hour)).toBe(expected);
  });
});

describe('localDateKey (the inverse of store.ts parseStatsDate)', () => {
  it('round-trips a local y/m/d Date back to its yyyy-MM-dd key regardless of host UTC offset', () => {
    expect(localDateKey(new Date(2026, 1, 17))).toBe('2026-02-17');
    expect(localDateKey(new Date(2026, 0, 1))).toBe('2026-01-01');
    expect(localDateKey(new Date(2025, 11, 31))).toBe('2025-12-31');
  });

  it('pads single-digit months and days', () => {
    expect(localDateKey(new Date(2026, 2, 5))).toBe('2026-03-05');
  });
});

// ADR 0010 control 1 / M3-33 AC2: ModelUsageChart.vue, CostBreakdownChart.vue and
// TokenUsageOverTimeChart.vue all call this on a model name (ultimately a raw JSONL `model`
// field, attacker-influenceable) before it reaches an ECharts tooltip -- real HTML, not a Vue
// template, so Vue's own auto-escaping does not apply there.
describe('escapeForTooltip (ADR 0010 control 1)', () => {
  it('neutralises a script tag so no raw markup survives', () => {
    const escaped = escapeForTooltip('<script>alert(1)</script>');
    expect(escaped).not.toContain('<script>');
    expect(escaped).not.toMatch(/[<>]/);
  });

  it('neutralises an img onerror payload', () => {
    const escaped = escapeForTooltip('<img src=x onerror="alert(document.cookie)">');
    expect(escaped).not.toMatch(/[<>]/);
  });

  it('leaves an ordinary model name unchanged', () => {
    expect(escapeForTooltip('Opus 4.6')).toBe('Opus 4.6');
  });
});
