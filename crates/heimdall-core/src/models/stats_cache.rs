//! `~/.claude/stats-cache.json` (Models/StatsCache.swift). The on-disk keys are camelCase, like the IPC shape.
//! Decoding is permissive: missing fields take defaults instead of failing the file.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Whether `get_stats` served the on-disk cache or recomputed it fresh (contract `StatsView`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum StatsSource {
    Cache,
    Computed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatsView {
    pub stats: StatsCache,
    pub source: StatsSource,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct StatsCache {
    pub version: u32,
    pub last_computed_date: String,
    pub daily_activity: Vec<DailyActivity>,
    pub daily_model_tokens: Vec<DailyModelTokens>,
    pub model_usage: BTreeMap<String, ModelUsage>,
    pub total_sessions: u64,
    pub total_messages: u64,
    pub longest_session: LongestSession,
    pub first_session_date: String,
    pub hour_counts: BTreeMap<String, u64>,
    pub total_speculation_time_saved_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct DailyActivity {
    pub date: String,
    pub message_count: u64,
    pub session_count: u64,
    pub tool_call_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct DailyModelTokens {
    pub date: String,
    pub tokens_by_model: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub web_search_requests: u64,
    /// Only Claude Code's own cache carries real costs; local recomputation writes 0 (ADR parity K2).
    #[serde(rename = "costUSD")]
    pub cost_usd: f64,
    pub context_window: u64,
    pub max_output_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct LongestSession {
    pub session_id: String,
    /// Milliseconds.
    pub duration: u64,
    pub message_count: u64,
    pub timestamp: String,
}

impl StatsCache {
    pub fn first_session_parsed_date(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.first_session_date).ok().map(|d| d.with_timezone(&Utc))
    }

    pub fn total_output_tokens(&self) -> u64 {
        self.model_usage.values().map(|m| m.output_tokens).sum()
    }

    /// Input plus cache reads and cache writes, across all models.
    pub fn total_input_tokens(&self) -> u64 {
        self.model_usage.values().map(|m| m.input_tokens + m.cache_read_input_tokens + m.cache_creation_input_tokens).sum()
    }

    pub fn total_cost_usd(&self) -> f64 {
        self.model_usage.values().map(|m| m.cost_usd).sum()
    }

    pub fn total_tool_calls(&self) -> u64 {
        self.daily_activity.iter().map(|d| d.tool_call_count).sum()
    }

    /// `(hour, count)` sorted by hour; keys that are not numbers are ignored.
    pub fn sorted_hour_counts(&self) -> Vec<(u32, u64)> {
        let mut v: Vec<_> = self.hour_counts.iter().filter_map(|(k, &c)| k.parse::<u32>().ok().map(|h| (h, c))).collect();
        v.sort_by_key(|&(h, _)| h);
        v
    }

    /// StatsCache.friendlyModelName: a short label for the models Poirot knew; anything else unchanged.
    pub fn friendly_model_name(model_id: &str) -> String {
        const KNOWN: [(&str, &str); 5] =
            [("opus-4-6", "Opus 4.6"), ("opus-4-5", "Opus 4.5"), ("sonnet-4-5", "Sonnet 4.5"), ("sonnet-4", "Sonnet 4"), ("haiku", "Haiku")];
        KNOWN.iter().find(|(needle, _)| model_id.contains(needle)).map_or_else(|| model_id.to_string(), |(_, label)| label.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    const VALID_JSON: &str = r#"{
      "version": 2, "lastComputedDate": "2026-02-17",
      "dailyActivity": [
        {"date": "2026-01-01", "messageCount": 18255, "sessionCount": 11, "toolCallCount": 5324},
        {"date": "2026-01-02", "messageCount": 360, "sessionCount": 1, "toolCallCount": 100}
      ],
      "dailyModelTokens": [
        {"date": "2026-01-01", "tokensByModel": {"claude-opus-4-5-20251101": 2269186}},
        {"date": "2026-01-12", "tokensByModel": {"claude-opus-4-5-20251101": 196417, "claude-sonnet-4-5-20250929": 99944}}
      ],
      "modelUsage": {
        "claude-opus-4-5-20251101": {"inputTokens": 4020186, "outputTokens": 3833943, "cacheReadInputTokens": 7246318082,
          "cacheCreationInputTokens": 325882562, "webSearchRequests": 0, "costUSD": 0, "contextWindow": 0, "maxOutputTokens": 0},
        "claude-opus-4-6": {"inputTokens": 592180, "outputTokens": 1778001, "cacheReadInputTokens": 2323328386,
          "cacheCreationInputTokens": 103861546, "webSearchRequests": 0, "costUSD": 0, "contextWindow": 0, "maxOutputTokens": 0}
      },
      "totalSessions": 466, "totalMessages": 322626,
      "longestSession": {"sessionId": "ce55f01b-e4f4-4c6a-934e-f95ba536935f", "duration": 321972719, "messageCount": 1504,
        "timestamp": "2026-01-13T19:55:30.879Z"},
      "firstSessionDate": "2025-12-31T19:57:52.149Z",
      "hourCounts": {"0": 3, "11": 52, "23": 16},
      "totalSpeculationTimeSavedMs": 0
    }"#;

    fn stats() -> StatsCache {
        serde_json::from_str(VALID_JSON).unwrap()
    }

    // Ported from the model cases of PoirotTests/Models/StatsCacheTests.swift
    // (longestSessionFormatted moved to Vitest; loader cases live in data/stats_cache.rs).
    #[test]
    fn decode_full_json_succeeds() {
        let s = stats();
        assert_eq!((s.version, s.last_computed_date.as_str(), s.total_sessions, s.total_messages, s.total_speculation_time_saved_ms),
                   (2, "2026-02-17", 466, 322_626, 0));
    }
    #[test]
    fn decode_daily_activity_parses_correctly() {
        let d = &stats().daily_activity;
        assert_eq!(d.len(), 2);
        assert_eq!(d[0], DailyActivity { date: "2026-01-01".into(), message_count: 18255, session_count: 11, tool_call_count: 5324 });
    }
    #[test]
    fn decode_daily_model_tokens_parses_multi_model() {
        let t = &stats().daily_model_tokens[1].tokens_by_model;
        assert_eq!(t.len(), 2);
        assert_eq!((t["claude-opus-4-5-20251101"], t["claude-sonnet-4-5-20250929"]), (196_417, 99_944));
    }
    #[test]
    fn decode_model_usage_parses_token_counts() {
        let s = stats();
        let opus = &s.model_usage["claude-opus-4-5-20251101"];
        assert_eq!(s.model_usage.len(), 2);
        assert_eq!((opus.input_tokens, opus.output_tokens, opus.cache_read_input_tokens, opus.cache_creation_input_tokens),
                   (4_020_186, 3_833_943, 7_246_318_082, 325_882_562));
    }
    #[test]
    fn decode_longest_session_parses_all_fields() {
        let l = stats().longest_session;
        assert_eq!((l.session_id.as_str(), l.duration, l.message_count), ("ce55f01b-e4f4-4c6a-934e-f95ba536935f", 321_972_719, 1504));
    }
    #[test]
    fn decode_hour_counts_parses_string_keys() {
        let h = stats().hour_counts;
        assert_eq!((h["0"], h["11"], h["23"]), (3, 52, 16));
    }
    #[test]
    fn sorted_hour_counts_sorts_ascending() {
        assert_eq!(stats().sorted_hour_counts().iter().map(|x| x.0).collect::<Vec<_>>(), [0, 11, 23]);
    }
    #[test]
    fn total_output_tokens_sums_across_models() {
        assert_eq!(stats().total_output_tokens(), 3_833_943 + 1_778_001);
    }
    #[test]
    fn friendly_model_name_maps_known_models() {
        assert_eq!(StatsCache::friendly_model_name("claude-opus-4-6"), "Opus 4.6");
        assert_eq!(StatsCache::friendly_model_name("claude-opus-4-5-20251101"), "Opus 4.5");
        assert_eq!(StatsCache::friendly_model_name("claude-sonnet-4-5-20250929"), "Sonnet 4.5");
        assert_eq!(StatsCache::friendly_model_name("unknown-model"), "unknown-model");
    }
    #[test]
    fn first_session_parsed_date_parses_iso8601() {
        let d = stats().first_session_parsed_date().unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (2025, 12, 31));
    }
    #[test]
    fn total_tool_calls_sums_across_days() {
        assert_eq!(stats().total_tool_calls(), 5324 + 100);
    }

    #[test]
    fn cost_usd_keeps_its_exact_key_and_missing_fields_default() {
        let json = serde_json::to_value(ModelUsage { cost_usd: 1.5, ..Default::default() }).unwrap();
        assert_eq!(json["costUSD"], 1.5);
        let partial: StatsCache = serde_json::from_str(r#"{"version": 3}"#).unwrap();
        assert_eq!((partial.version, partial.daily_activity.len()), (3, 0));
    }
}
