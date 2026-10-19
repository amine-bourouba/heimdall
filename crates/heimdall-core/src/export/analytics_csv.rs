//! Analytics CSV export (Views/Analytics/Export/AnalyticsCSVExporter.swift).

use std::collections::BTreeSet;

use super::CsvKind;
use crate::models::stats_cache::StatsCache;

/// `AnalyticsCSVExporter.export(_:type:)`, ported one table at a time below.
pub fn export(stats: &StatsCache, kind: CsvKind) -> String {
    match kind {
        CsvKind::DailyActivity => daily_activity(stats),
        CsvKind::ModelUsage => model_usage(stats),
        CsvKind::DailyTokens => daily_tokens(stats),
        CsvKind::HourlyActivity => hourly_activity(stats),
    }
}

/// `AnalyticsExportType.suggestedFileName`.
pub fn suggested_file_name(kind: CsvKind) -> &'static str {
    match kind {
        CsvKind::DailyActivity => "daily-activity.csv",
        CsvKind::ModelUsage => "model-usage.csv",
        CsvKind::DailyTokens => "daily-tokens.csv",
        CsvKind::HourlyActivity => "hourly-activity.csv",
    }
}

fn daily_activity(stats: &StatsCache) -> String {
    let mut csv = "Date,Messages,Sessions,Tool Calls\n".to_string();
    for day in &stats.daily_activity {
        csv += &format!(
            "{},{},{},{}\n",
            day.date, day.message_count, day.session_count, day.tool_call_count
        );
    }
    csv
}

/// Sorted by cost descending (`modelUsage.sorted(by: { $0.value.costUSD > $1.value.costUSD })`);
/// `model_usage` is a `BTreeMap` (alphabetical by model id), so a tie in cost breaks
/// alphabetically -- a real, deterministic order Swift's own `[String: ModelUsage]` dictionary
/// cannot promise at all, not something to match byte-for-byte since nothing in the carried-over
/// tests' own data has a tie.
fn model_usage(stats: &StatsCache) -> String {
    let mut csv = "Model,Input Tokens,Output Tokens,Cache Read Tokens,Cache Creation Tokens,Cost USD\n".to_string();
    let mut entries: Vec<_> = stats.model_usage.iter().collect();
    entries.sort_by(|a, b| {
        b.1.cost_usd
            .partial_cmp(&a.1.cost_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (model_id, usage) in entries {
        let name = StatsCache::friendly_model_name(model_id);
        csv += &format!(
            "{name},{},{},{},{},{:.2}\n",
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_input_tokens,
            usage.cache_creation_input_tokens,
            usage.cost_usd
        );
    }
    csv
}

fn daily_tokens(stats: &StatsCache) -> String {
    let all_models: BTreeSet<&str> = stats
        .daily_model_tokens
        .iter()
        .flat_map(|d| d.tokens_by_model.keys().map(String::as_str))
        .collect();
    let friendly_names: Vec<String> = all_models.iter().map(|m| StatsCache::friendly_model_name(m)).collect();
    let mut csv = format!("Date,{}\n", friendly_names.join(","));
    for day in &stats.daily_model_tokens {
        let values: Vec<String> = all_models
            .iter()
            .map(|m| day.tokens_by_model.get(*m).copied().unwrap_or(0).to_string())
            .collect();
        csv += &format!("{},{}\n", day.date, values.join(","));
    }
    csv
}

fn hourly_activity(stats: &StatsCache) -> String {
    let mut csv = "Hour,Sessions\n".to_string();
    for (hour, count) in stats.sorted_hour_counts() {
        csv += &format!("{hour},{count}\n");
    }
    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::stats_cache::{DailyActivity, DailyModelTokens, LongestSession, ModelUsage};

    // Ported from PoirotTests/Views/AnalyticsCSVExporterTests.swift.
    fn stats() -> StatsCache {
        StatsCache {
            version: 2,
            last_computed_date: "2026-02-17".to_string(),
            daily_activity: vec![
                DailyActivity {
                    date: "2026-01-01".to_string(),
                    message_count: 100,
                    session_count: 5,
                    tool_call_count: 50,
                },
                DailyActivity {
                    date: "2026-01-02".to_string(),
                    message_count: 200,
                    session_count: 8,
                    tool_call_count: 120,
                },
            ],
            daily_model_tokens: vec![
                DailyModelTokens {
                    date: "2026-01-01".to_string(),
                    tokens_by_model: [
                        ("claude-opus-4-5-20251101".to_string(), 1000),
                        ("claude-opus-4-6".to_string(), 500),
                    ]
                    .into(),
                },
                DailyModelTokens {
                    date: "2026-01-02".to_string(),
                    tokens_by_model: [("claude-opus-4-5-20251101".to_string(), 2000)].into(),
                },
            ],
            model_usage: [
                (
                    "claude-opus-4-5-20251101".to_string(),
                    ModelUsage {
                        input_tokens: 4000,
                        output_tokens: 3000,
                        cache_read_input_tokens: 7000,
                        cache_creation_input_tokens: 300,
                        web_search_requests: 0,
                        cost_usd: 15.50,
                        context_window: 200_000,
                        max_output_tokens: 32000,
                    },
                ),
                (
                    "claude-opus-4-6".to_string(),
                    ModelUsage {
                        input_tokens: 800,
                        output_tokens: 1500,
                        cache_read_input_tokens: 200,
                        cache_creation_input_tokens: 100,
                        web_search_requests: 0,
                        cost_usd: 10.25,
                        context_window: 200_000,
                        max_output_tokens: 32000,
                    },
                ),
            ]
            .into(),
            total_sessions: 10,
            total_messages: 300,
            longest_session: LongestSession {
                session_id: "test".to_string(),
                duration: 3_600_000,
                message_count: 50,
                timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            },
            first_session_date: "2025-12-31T00:00:00.000Z".to_string(),
            hour_counts: [("9".to_string(), 5), ("14".to_string(), 10), ("22".to_string(), 3)].into(),
            total_speculation_time_saved_ms: 0,
        }
    }

    fn lines(csv: &str) -> Vec<&str> {
        csv.split('\n').collect()
    }
    fn non_empty_lines(csv: &str) -> Vec<&str> {
        csv.split('\n').filter(|l| !l.is_empty()).collect()
    }

    #[test]
    fn daily_activity_has_correct_header() {
        assert_eq!(
            lines(&export(&stats(), CsvKind::DailyActivity))[0],
            "Date,Messages,Sessions,Tool Calls"
        );
    }
    #[test]
    fn daily_activity_has_correct_row_count() {
        assert_eq!(non_empty_lines(&export(&stats(), CsvKind::DailyActivity)).len(), 3);
    }
    #[test]
    fn daily_activity_has_correct_values() {
        let csv = export(&stats(), CsvKind::DailyActivity);
        let l = lines(&csv);
        assert_eq!(l[1], "2026-01-01,100,5,50");
        assert_eq!(l[2], "2026-01-02,200,8,120");
    }

    #[test]
    fn model_usage_has_correct_header() {
        assert_eq!(
            lines(&export(&stats(), CsvKind::ModelUsage))[0],
            "Model,Input Tokens,Output Tokens,Cache Read Tokens,Cache Creation Tokens,Cost USD"
        );
    }
    #[test]
    fn model_usage_sorted_by_cost_descending() {
        let csv = export(&stats(), CsvKind::ModelUsage);
        assert!(non_empty_lines(&csv)[1].starts_with("Opus 4.5"));
    }

    #[test]
    fn daily_tokens_has_model_columns_in_header() {
        let csv = export(&stats(), CsvKind::DailyTokens);
        let header = lines(&csv)[0];
        assert!(header.starts_with("Date,"));
        assert!(header.contains("Opus"));
    }
    #[test]
    fn daily_tokens_has_missing_models_as_zero() {
        let csv = export(&stats(), CsvKind::DailyTokens);
        let l = non_empty_lines(&csv);
        // Second data row: opus-4-6 has no tokens for 2026-01-02 -> should be 0.
        assert!(l[2].contains('0'));
    }

    #[test]
    fn hourly_activity_has_correct_header() {
        assert_eq!(lines(&export(&stats(), CsvKind::HourlyActivity))[0], "Hour,Sessions");
    }
    #[test]
    fn hourly_activity_sorted_by_hour() {
        let csv = export(&stats(), CsvKind::HourlyActivity);
        let l = non_empty_lines(&csv);
        assert_eq!(l[1], "9,5");
        assert_eq!(l[2], "14,10");
        assert_eq!(l[3], "22,3");
    }

    #[test]
    fn empty_daily_activity_only_header() {
        let empty = StatsCache::default();
        let csv = export(&empty, CsvKind::DailyActivity);
        let l = non_empty_lines(&csv);
        assert_eq!(l.len(), 1);
        assert_eq!(l[0], "Date,Messages,Sessions,Tool Calls");
    }

    #[test]
    fn export_type_suggested_file_names() {
        assert_eq!(suggested_file_name(CsvKind::DailyActivity), "daily-activity.csv");
        assert_eq!(suggested_file_name(CsvKind::ModelUsage), "model-usage.csv");
        assert_eq!(suggested_file_name(CsvKind::DailyTokens), "daily-tokens.csv");
        assert_eq!(suggested_file_name(CsvKind::HourlyActivity), "hourly-activity.csv");
    }
}
