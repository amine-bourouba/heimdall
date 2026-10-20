//! Local stats recomputation (Services/StatsComputer.swift): a [`StatsCache`] built from the
//! transcripts themselves, for when Claude Code's `stats-cache.json` is missing or stale.
//!
//! Only main sessions (UUID-named transcripts, non-sidechain records) are read, so sub-agent
//! usage is left out as in the Swift reference app. Days and hours are in the given time zone (local in the app).
//! Token usage counts each API message once, from its last record (D5); the Swift reference app added every
//! record's copy of the usage.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, SecondsFormat, TimeZone, Timelike, Utc};
use serde::Deserialize;
use serde::de::{Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::Value;

use crate::jsonl;
use crate::models::message::SYNTHETIC_MODEL;
use crate::models::stats_cache::{DailyActivity, DailyModelTokens, LongestSession, ModelUsage, StatsCache};
use crate::projects::discovery::is_uuid;
use crate::transcript::blocks::int_value;

/// The transcript fields stats need; the rest of each line is skipped unread.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Record {
    #[serde(rename = "type")]
    kind: Option<String>,
    is_sidechain: Option<bool>,
    timestamp: Option<String>,
    message: Option<Message>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Message {
    id: Option<String>,
    model: Option<String>,
    usage: Option<Usage>,
    #[serde(deserialize_with = "tool_use_count")]
    content: u64,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
struct Usage {
    input_tokens: Option<Value>,
    output_tokens: Option<Value>,
    cache_read_input_tokens: Option<Value>,
    cache_creation_input_tokens: Option<Value>,
    server_tool_use: Option<ServerToolUse>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
struct ServerToolUse {
    web_search_requests: Option<Value>,
}

#[derive(Deserialize)]
struct BlockKind {
    #[serde(rename = "type")]
    kind: Option<String>,
}

/// Counts `tool_use` blocks in a message's `content` without keeping it; a string or any other
/// shape counts zero.
fn tool_use_count<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    struct Count;
    impl<'de> Visitor<'de> for Count {
        type Value = u64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("message content")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<u64, A::Error> {
            let mut n = 0;
            while let Some(block) = seq.next_element::<Option<BlockKind>>()? {
                n += u64::from(block.and_then(|b| b.kind).as_deref() == Some("tool_use"));
            }
            Ok(n)
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<u64, A::Error> {
            while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
            Ok(0)
        }
        fn visit_str<E>(self, _: &str) -> Result<u64, E> {
            Ok(0)
        }
        fn visit_unit<E>(self) -> Result<u64, E> {
            Ok(0)
        }
        fn visit_bool<E>(self, _: bool) -> Result<u64, E> {
            Ok(0)
        }
        fn visit_i64<E>(self, _: i64) -> Result<u64, E> {
            Ok(0)
        }
        fn visit_u64<E>(self, _: u64) -> Result<u64, E> {
            Ok(0)
        }
        fn visit_f64<E>(self, _: f64) -> Result<u64, E> {
            Ok(0)
        }
    }
    d.deserialize_any(Count)
}

/// Stats from every main session under `projects_dir`, in local time. `None` when there is none.
pub fn compute(projects_dir: &Path) -> Option<StatsCache> {
    compute_in(projects_dir, &Local)
}

pub fn compute_in<Tz: TimeZone>(projects_dir: &Path, tz: &Tz) -> Option<StatsCache>
where
    Tz::Offset: Display,
{
    let mut agg = Aggregate::default();
    for file in session_files(projects_dir)? {
        agg.scan(&file, tz);
    }
    agg.finish(tz)
}

/// Local day (`YYYY-MM-DD`) of the most recently modified main transcript, to tell whether the
/// cache covers it without parsing anything.
pub fn latest_session_day(projects_dir: &Path) -> Option<String> {
    latest_session_day_in(projects_dir, &Local)
}

pub fn latest_session_day_in<Tz: TimeZone>(projects_dir: &Path, tz: &Tz) -> Option<String>
where
    Tz::Offset: Display,
{
    let latest = session_files(projects_dir)?
        .iter()
        .filter_map(|f| f.metadata().and_then(|m| m.modified()).ok())
        .max()?;
    Some(
        DateTime::<Utc>::from(latest)
            .with_timezone(tz)
            .format("%Y-%m-%d")
            .to_string(),
    )
}

/// UUID-named `.jsonl` files in each project folder (any letter case, as Swift's UUID parser).
fn session_files(projects_dir: &Path) -> Option<Vec<PathBuf>> {
    let mut files = Vec::new();
    for dir in std::fs::read_dir(projects_dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
    {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        files.extend(entries.flatten().map(|e| e.path()).filter(|p| {
            p.extension().is_some_and(|x| x == "jsonl")
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| is_uuid(&s.to_ascii_lowercase()))
        }));
    }
    files.sort();
    Some(files)
}

#[derive(Default)]
struct Day {
    messages: u64,
    sessions: u64,
    tool_calls: u64,
}

#[derive(Default)]
struct Aggregate {
    days: BTreeMap<String, Day>,
    day_model_tokens: BTreeMap<String, BTreeMap<String, u64>>,
    models: BTreeMap<String, ModelUsage>,
    hours: BTreeMap<u32, u64>,
    counted_ids: HashSet<String>,
    total_sessions: u64,
    total_messages: u64,
    earliest: Option<DateTime<Utc>>,
    longest: Option<LongestSession>,
}

impl Aggregate {
    fn scan<Tz: TimeZone>(&mut self, file: &Path, tz: &Tz)
    where
        Tz::Offset: Display,
    {
        let Ok(records) = jsonl::open::<Record>(file) else {
            return;
        };
        let session_id = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let (mut start, mut end): (Option<DateTime<Utc>>, Option<DateTime<Utc>>) = (None, None);
        let mut messages = 0;
        // Last usage per message id, with the day and model it belongs to.
        let mut usage_by_id: HashMap<String, (String, String, Usage)> = HashMap::new();

        for (n, record) in records.enumerate() {
            let Some(kind @ ("user" | "assistant")) = record.kind.as_deref() else {
                continue;
            };
            if record.is_sidechain == Some(true) {
                continue;
            }
            let message = record.message.unwrap_or_default();
            if kind == "assistant" && message.model.as_deref() == Some(SYNTHETIC_MODEL) {
                continue;
            }
            let Some(ts) = record
                .timestamp
                .as_deref()
                .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
                .map(|t| t.to_utc())
            else {
                continue;
            };
            messages += 1;
            self.total_messages += 1;
            start = Some(start.map_or(ts, |s| s.min(ts)));
            end = Some(end.map_or(ts, |e| e.max(ts)));
            self.earliest = Some(self.earliest.map_or(ts, |e| e.min(ts)));

            let local = ts.with_timezone(tz);
            let day = local.format("%Y-%m-%d").to_string();
            *self.hours.entry(local.hour()).or_default() += 1;
            let entry = self.days.entry(day.clone()).or_default();
            entry.messages += 1;
            if kind == "assistant" {
                entry.tool_calls += message.content;
                if let Some(usage) = message.usage {
                    let id = message.id.unwrap_or_else(|| format!("{session_id}:{n}"));
                    usage_by_id.insert(id, (day, message.model.unwrap_or_else(|| "unknown".into()), usage));
                }
            }
        }

        for (id, (day, model, usage)) in usage_by_id {
            // A message copied into another transcript (a resumed session) is counted once.
            if self.counted_ids.insert(id) {
                self.add_usage(&day, &model, &usage);
            }
        }
        let (Some(start), Some(end)) = (start, end) else { return };
        self.total_sessions += 1;
        self.days
            .entry(start.with_timezone(tz).format("%Y-%m-%d").to_string())
            .or_default()
            .sessions += 1;
        let duration = u64::try_from((end - start).num_milliseconds()).unwrap_or(0);
        if self.longest.as_ref().is_none_or(|l| duration > l.duration) {
            self.longest = Some(LongestSession {
                session_id,
                duration,
                message_count: messages,
                timestamp: iso(start),
            });
        }
    }

    fn add_usage(&mut self, day: &str, model: &str, usage: &Usage) {
        let (input, output) = (
            int_value(usage.input_tokens.as_ref()),
            int_value(usage.output_tokens.as_ref()),
        );
        let (cache_read, cache_creation) = (
            int_value(usage.cache_read_input_tokens.as_ref()),
            int_value(usage.cache_creation_input_tokens.as_ref()),
        );
        let web_search = int_value(
            usage
                .server_tool_use
                .as_ref()
                .and_then(|s| s.web_search_requests.as_ref()),
        );
        let m = self.models.entry(model.to_string()).or_default();
        m.input_tokens += input;
        m.output_tokens += output;
        m.cache_read_input_tokens += cache_read;
        m.cache_creation_input_tokens += cache_creation;
        m.web_search_requests += web_search;
        *self
            .day_model_tokens
            .entry(day.to_string())
            .or_default()
            .entry(model.to_string())
            .or_default() += input + output + cache_read + cache_creation;
    }

    fn finish<Tz: TimeZone>(self, tz: &Tz) -> Option<StatsCache>
    where
        Tz::Offset: Display,
    {
        if self.total_sessions == 0 {
            return None;
        }
        let (earliest, longest) = (self.earliest?, self.longest?);
        let model_usage = self
            .models
            .into_iter()
            .map(|(name, mut m)| {
                // Subscription usage has no per-call cost; the dashboard shows "included".
                m.cost_usd = 0.0;
                m.context_window =
                    if m.input_tokens + m.cache_read_input_tokens + m.cache_creation_input_tokens > 200_000 {
                        1_000_000
                    } else {
                        200_000
                    };
                m.max_output_tokens = 32_000;
                (name, m)
            })
            .collect();
        Some(StatsCache {
            version: 1,
            last_computed_date: Utc::now().with_timezone(tz).format("%Y-%m-%d").to_string(),
            daily_activity: self
                .days
                .into_iter()
                .map(|(date, d)| DailyActivity {
                    date,
                    message_count: d.messages,
                    session_count: d.sessions,
                    tool_call_count: d.tool_calls,
                })
                .collect(),
            daily_model_tokens: self
                .day_model_tokens
                .into_iter()
                .map(|(date, tokens_by_model)| DailyModelTokens { date, tokens_by_model })
                .collect(),
            model_usage,
            total_sessions: self.total_sessions,
            total_messages: self.total_messages,
            longest_session: longest,
            first_session_date: iso(earliest),
            hour_counts: self.hours.into_iter().map(|(h, c)| (h.to_string(), c)).collect(),
            total_speculation_time_saved_ms: 0,
        })
    }
}

fn iso(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use chrono::FixedOffset;

    pub(crate) fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }

    /// `<root>/<project>/<SESSION>.jsonl`; the session id is upper case, as Swift's UUID() wrote it.
    pub(crate) fn projects_with(jsonl: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("-Users-test-proj");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("0B6F5A52-7C3E-4F0A-9D1B-2E8C4A6F1D3B.jsonl"), jsonl).unwrap();
        root
    }

    // Ported from PoirotTests/Services/StatsComputerTests.swift.
    #[test]
    fn compute_aggregates_session_from_transcript() {
        let root = projects_with(concat!(
            r#"{"type":"user","timestamp":"2026-06-20T10:00:00.000Z","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-06-20T10:00:05.000Z","message":{"id":"a1","model":"claude-opus-4-6","usage":{"input_tokens":10,"output_tokens":20,"cache_read_input_tokens":1000,"cache_creation_input_tokens":100,"server_tool_use":{"web_search_requests":2}},"content":[{"type":"tool_use","name":"Read"},{"type":"text","text":"ok"}]}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-06-20T10:01:00.000Z","message":{"id":"a2","model":"claude-opus-4-6","usage":{"input_tokens":5,"output_tokens":8,"cache_read_input_tokens":2000,"cache_creation_input_tokens":50},"content":[{"type":"tool_use","name":"Bash"}]}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-06-20T10:02:00.000Z","isSidechain":true,"message":{"id":"s1","model":"claude-opus-4-6","usage":{"input_tokens":999,"output_tokens":999},"content":[]}}"#,
        ));
        let s = compute_in(root.path(), &utc()).unwrap();
        assert_eq!((s.total_sessions, s.total_messages), (1, 3));
        let m = &s.model_usage["claude-opus-4-6"];
        assert_eq!(
            (
                m.input_tokens,
                m.output_tokens,
                m.cache_read_input_tokens,
                m.cache_creation_input_tokens,
                m.web_search_requests
            ),
            (15, 28, 3000, 150, 2)
        );
        assert_eq!(m.cost_usd, 0.0);
        let d = &s.daily_activity;
        assert_eq!(
            (
                d.len(),
                d[0].message_count,
                d[0].session_count,
                d[0].tool_call_count,
                s.total_tool_calls()
            ),
            (1, 3, 1, 2, 2)
        );
        assert_eq!(
            (s.longest_session.duration, s.longest_session.message_count),
            (60_000, 3)
        );
    }
    #[test]
    fn compute_no_sessions_returns_nil() {
        let root = tempfile::tempdir().unwrap();
        assert!(compute_in(root.path(), &utc()).is_none());
    }
    #[test]
    fn compute_missing_path_returns_nil() {
        assert!(compute_in(Path::new("/no/such/dir/4b1d"), &utc()).is_none());
    }
    #[test]
    fn compute_includes_tokens_on_later_days() {
        let root = projects_with(concat!(
            r#"{"type":"user","timestamp":"2026-03-15T10:00:00.000Z","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-03-15T10:00:05.000Z","message":{"id":"a1","model":"claude-opus-4-6","usage":{"input_tokens":10,"output_tokens":5},"content":[]}}"#,
            "\n",
            r#"{"type":"user","timestamp":"2026-08-20T10:00:00.000Z","uuid":"u2","message":{"role":"user","content":"later"}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-08-20T10:00:05.000Z","message":{"id":"a2","model":"claude-opus-4-6","usage":{"input_tokens":40,"output_tokens":20},"content":[]}}"#,
        ));
        let s = compute_in(root.path(), &utc()).unwrap();
        let tokens = |date: &str| {
            s.daily_model_tokens
                .iter()
                .find(|d| d.date == date)
                .unwrap()
                .tokens_by_model["claude-opus-4-6"]
        };
        assert_eq!((tokens("2026-03-15"), tokens("2026-08-20")), (15, 60));
    }
    #[test]
    fn compute_accepts_floating_point_token_counts() {
        let root = projects_with(concat!(
            r#"{"type":"user","timestamp":"2026-08-20T10:00:00.000Z","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-08-20T10:00:05.000Z","message":{"id":"a1","model":"claude-opus-4-6","usage":{"input_tokens":10.0,"output_tokens":5.0,"cache_read_input_tokens":100.0},"content":[]}}"#,
        ));
        let m = &compute_in(root.path(), &utc()).unwrap().model_usage["claude-opus-4-6"];
        assert_eq!(
            (m.input_tokens, m.output_tokens, m.cache_read_input_tokens),
            (10, 5, 100)
        );
    }
    #[test]
    fn int_value_reads_ns_number_and_ignores_bool() {
        use serde_json::json;
        assert_eq!(
            [json!(42), json!(7.0), json!(true), json!("nope")].map(|v| int_value(Some(&v))),
            [42, 7, 0, 0]
        );
        assert_eq!(int_value(None), 0);
    }

    #[test]
    fn a_message_split_over_records_counts_its_last_usage_once() {
        let root = projects_with(concat!(
            r#"{"type":"user","timestamp":"2026-08-20T10:00:00.000Z","message":{"content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-08-20T10:00:01.000Z","message":{"id":"m","model":"x","usage":{"input_tokens":2,"output_tokens":8,"cache_read_input_tokens":500},"content":[{"type":"thinking"}]}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-08-20T10:00:02.000Z","message":{"id":"m","model":"x","usage":{"input_tokens":2,"output_tokens":8,"cache_read_input_tokens":500},"content":[{"type":"tool_use"}]}}"#,
            "\n",
            r#"{"type":"user","timestamp":"2026-08-20T10:00:03.000Z","message":{"content":[{"type":"tool_result","content":"ok"}]}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-08-20T10:00:04.000Z","message":{"id":"m","model":"x","usage":{"input_tokens":2,"output_tokens":209,"cache_read_input_tokens":500},"content":[{"type":"tool_use"}]}}"#,
        ));
        let s = compute_in(root.path(), &utc()).unwrap();
        let m = &s.model_usage["x"];
        // the Swift reference app would report 6 input, 225 output and 1500 cache reads.
        assert_eq!(
            (m.input_tokens, m.output_tokens, m.cache_read_input_tokens),
            (2, 209, 500)
        );
        assert_eq!((s.total_messages, s.total_tool_calls()), (5, 2));
    }

    #[test]
    fn days_and_hours_follow_the_time_zone() {
        let root =
            projects_with(r#"{"type":"user","timestamp":"2026-08-20T23:30:00.000Z","message":{"content":"late"}}"#);
        let plus_two = FixedOffset::east_opt(2 * 3600).unwrap();
        let s = compute_in(root.path(), &plus_two).unwrap();
        assert_eq!(
            (s.daily_activity[0].date.as_str(), s.sorted_hour_counts()),
            ("2026-08-21", vec![(1, 1)])
        );
        assert_eq!(s.first_session_date, "2026-08-20T23:30:00.000Z");
    }

    #[test]
    fn windows_fixture_recomputes_without_error() {
        let projects = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/windows/claude-home/projects");
        let s = compute_in(&projects, &utc()).unwrap();
        assert!(s.total_sessions >= 5 && s.total_output_tokens() > 0);
        assert!(s.daily_activity.windows(2).all(|w| w[0].date < w[1].date));
    }
}
