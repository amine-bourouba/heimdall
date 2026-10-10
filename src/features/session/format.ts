// Ported from Session.swift (`formatModel`/`formatTokens`/`timeAgo`, SessionsNavigationView.swift
// lines 750-773). `timeAgo` used Foundation's `RelativeDateTimeFormatter`; the web has the same
// capability natively (`Intl.RelativeTimeFormat`) -- used directly rather than hand-rolled, same
// "native platform feature over custom code" choice as everywhere else in this port.
import type { TokenUsage } from '@/ipc/bindings';

const RELATIVE_TIME = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
const RELATIVE_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 31536000],
  ['month', 2592000],
  ['week', 604800],
  ['day', 86400],
  ['hour', 3600],
  ['minute', 60],
];

export function formatRelativeTime(iso: string, now: Date = new Date()): string {
  const diffSeconds = (new Date(iso).getTime() - now.getTime()) / 1000;
  for (const [unit, secondsInUnit] of RELATIVE_UNITS) {
    if (Math.abs(diffSeconds) >= secondsInUnit) return RELATIVE_TIME.format(Math.round(diffSeconds / secondsInUnit), unit);
  }
  return RELATIVE_TIME.format(Math.round(diffSeconds), 'second');
}

/** "claude-opus-4-5" -> "Opus 4.5", "claude-sonnet-4-5-20260101" -> "Sonnet 4.5", anything else
 * returned unchanged -- ported verbatim from `SessionsListRow.formatModel`. */
export function formatModel(model: string): string {
  const name = model.replace('claude-', '');
  for (const [prefix, label] of [
    ['opus-', 'Opus'],
    ['sonnet-', 'Sonnet'],
    ['haiku-', 'Haiku'],
  ] as const) {
    if (name.startsWith(prefix)) {
      const version = name.slice(prefix.length).split(/[-_]/)[0];
      return `${label} ${version}`;
    }
  }
  return model;
}

/** 12500 -> "12.5k", 12000 -> "12k", 500 -> "500" -- ported verbatim from
 * `SessionsListRow.formatTokens`. */
export function formatTokenCount(count: number): string {
  if (count < 1000) return String(count);
  const k = count / 1000;
  return Number.isInteger(k) ? `${k}k` : `${k.toFixed(1)}k`;
}

export function totalTokens(usage: TokenUsage): number {
  return usage.input + usage.output + usage.cacheRead + usage.cacheCreation;
}
