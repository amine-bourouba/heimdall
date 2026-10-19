//! `history.jsonl` deletes through the safe-write rules (ADR 0009's own rules extended to a
//! line-oriented file, since this is never parsed as one JSON document): a line is matched back
//! to a given id the same way [`super::history::load_all`] computed it when the id was handed
//! out, every kept line is written back byte for byte (including its own line ending), and the
//! write refuses if the file changed on disk since it was read -- [`json_doc`](crate::config::write::json_doc)'s
//! own optimistic-concurrency check, ported to a file this crate never builds a `JsonDoc` for.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

use serde_json::Value;

use super::history::entry_id;
use crate::config::write::atomic;
use crate::error::{AppError, Result, WriteOutcome};

const BOM: char = '\u{feff}';

/// Removes the one line whose id is `entry_id_to_delete`. Not found is `Unchanged`, not an error
/// -- the entry may already have been removed by a previous call.
pub fn delete_entry(history_file: &Path, entry_id_to_delete: &str) -> Result<WriteOutcome> {
    rewrite_keeping(history_file, |id, _ts_ms| id != entry_id_to_delete)
}

/// Removes every line older than `days` days, measured from `now_ms` (epoch milliseconds) --
/// passed in rather than read from the clock so this stays deterministic to test; the command
/// layer (M3-46) supplies the real time.
pub fn clear_older_than(history_file: &Path, days: u32, now_ms: i64) -> Result<WriteOutcome> {
    let cutoff_ms = now_ms - i64::from(days) * 86_400_000;
    rewrite_keeping(history_file, |_id, ts_ms| ts_ms >= cutoff_ms)
}

/// Rewrites `path`, dropping every line `keep` returns `false` for. A line that is not valid JSON
/// or has no numeric `timestamp` is never judged -- always kept, exactly like `history::load_all`'s
/// own `skipped` lines, which this never turns into something it then deletes. Blank lines are
/// kept too and never counted into the repeat-occurrence numbering `entry_id` needs, matching
/// `load_all` exactly so the same line always gets the same id here as it did when `load_all`
/// handed that id to the caller in the first place.
fn rewrite_keeping(path: &Path, keep: impl FnMut(&str, i64) -> bool) -> Result<WriteOutcome> {
    let original = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(WriteOutcome::Unchanged),
        Err(e) => return Err(AppError::from_io(&e, path)),
    };
    rewrite_keeping_from(path, &original, keep)
}

/// [`rewrite_keeping`] split out so a test can supply a deliberately stale `original` snapshot and
/// exercise the busy path directly, without a second thread racing a real write into the tiny
/// window between this function's own two reads.
fn rewrite_keeping_from(path: &Path, original: &[u8], mut keep: impl FnMut(&str, i64) -> bool) -> Result<WriteOutcome> {
    let text = String::from_utf8(original.to_vec())
        .map_err(|_| AppError::parse("history.jsonl is not valid UTF-8, so it was left untouched").with_path(path))?;

    let mut repeats: HashMap<u64, u32> = HashMap::new();
    let mut kept: Vec<&str> = Vec::new();
    let mut removed_any = false;
    for (i, raw_line) in text.split('\n').enumerate() {
        // `BufRead::lines()` (what `load_all` reads with) strips a trailing `\r` and the first
        // line's BOM before anything ever sees the line's content; replicated here only to
        // compute the matching hash/id, never applied to `raw_line` itself, which is what gets
        // written back so a kept line's own bytes -- CRLF, BOM and all -- never change.
        let content = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let content = if i == 0 {
            content.strip_prefix(BOM).unwrap_or(content)
        } else {
            content
        };
        let should_keep = if content.trim().is_empty() {
            true
        } else {
            match line_timestamp_ms(content) {
                Some(ts_ms) => {
                    let occurrence = repeats.entry(crate::stable_hash(content.as_bytes())).or_default();
                    let id = entry_id(content, ts_ms, *occurrence);
                    *occurrence += 1;
                    keep(&id, ts_ms)
                }
                None => true,
            }
        };
        if should_keep {
            kept.push(raw_line);
        } else {
            removed_any = true;
        }
    }
    if !removed_any {
        return Ok(WriteOutcome::Unchanged);
    }

    // `JsonDoc::save`'s own optimistic-concurrency check, ported: re-read right before writing and
    // refuse if the file is no longer what was read at the top of this function.
    let current = std::fs::read(path).map_err(|e| AppError::from_io(&e, path))?;
    if current != original {
        return Err(AppError::busy("the file changed since it was read; reload and try again").with_path(path));
    }
    atomic::replace(path, kept.join("\n").as_bytes())?;
    Ok(WriteOutcome::Written)
}

/// The raw `timestamp` field a history line carries, as `history::RawEntry` reads it -- re-parsed
/// minimally here (that struct is private to `history.rs`) rather than building a full
/// `HistoryEntry` just to throw most of it away.
fn line_timestamp_ms(line: &str) -> Option<i64> {
    serde_json::from_str::<Value>(line)
        .ok()?
        .get("timestamp")?
        .as_f64()
        .map(|ms| ms as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::history::load_all;

    fn line(display: &str, ts: i64, project: &str) -> String {
        format!(r#"{{"display":"{display}","pastedContents":{{}},"timestamp":{ts},"project":"{project}"}}"#)
    }
    fn file(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }
    fn displays(path: &Path) -> Vec<String> {
        load_all(path).unwrap().value.into_iter().map(|e| e.display).collect()
    }

    #[test]
    fn delete_entry_removes_only_the_matching_line() {
        let (_d, p) = file(&[line("a", 1000, "/x"), line("b", 2000, "/x"), line("c", 3000, "/x")].join("\n"));
        let id = load_all(&p)
            .unwrap()
            .value
            .into_iter()
            .find(|e| e.display == "b")
            .unwrap()
            .id;
        assert_eq!(delete_entry(&p, &id).unwrap(), WriteOutcome::Written);
        assert_eq!(displays(&p), ["c", "a"]);
    }

    #[test]
    fn delete_entry_on_the_nth_repeat_of_an_identical_line_removes_only_that_one() {
        let dup = line("same", 1000, "/x");
        let (_d, p) = file(&format!("{dup}\n{dup}\n"));
        let entries = load_all(&p).unwrap().value;
        let (first, second) = (&entries[0].id, &entries[1].id);
        assert_ne!(first, second);
        assert_eq!(delete_entry(&p, second).unwrap(), WriteOutcome::Written);
        let remaining = load_all(&p).unwrap().value;
        assert_eq!((remaining.len(), remaining[0].id.as_str()), (1, first.as_str()));
    }

    #[test]
    fn delete_entry_unknown_id_is_unchanged() {
        let (_d, p) = file(&line("a", 1000, "/x"));
        let before = std::fs::read(&p).unwrap();
        assert_eq!(delete_entry(&p, "no-such-id").unwrap(), WriteOutcome::Unchanged);
        assert_eq!(std::fs::read(&p).unwrap(), before);
    }

    #[test]
    fn delete_entry_missing_file_is_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("history.jsonl");
        assert_eq!(delete_entry(&p, "anything").unwrap(), WriteOutcome::Unchanged);
    }

    #[test]
    fn delete_entry_keeps_an_unparseable_line_untouched() {
        let (_d, p) = file(&format!("not valid json\n{}\n", line("a", 1000, "/x")));
        let id = load_all(&p).unwrap().value[0].id.clone();
        delete_entry(&p, &id).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "not valid json\n");
    }

    #[test]
    fn delete_entry_preserves_blank_lines_and_crlf_of_untouched_lines() {
        let a = line("a", 1000, "/x");
        let b = line("b", 2000, "/x");
        let (_d, p) = file(&format!("{a}\r\n\r\n{b}\r\n"));
        let id = load_all(&p)
            .unwrap()
            .value
            .into_iter()
            .find(|e| e.display == "b")
            .unwrap()
            .id;
        delete_entry(&p, &id).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), format!("{a}\r\n\r\n"));
    }

    #[test]
    fn delete_entry_refuses_if_the_file_changed_since_it_was_read() {
        let (_d, p) = file(&[line("a", 1000, "/x"), line("b", 2000, "/x")].join("\n"));
        let id = load_all(&p)
            .unwrap()
            .value
            .into_iter()
            .find(|e| e.display == "b")
            .unwrap()
            .id;
        let stale_original = std::fs::read(&p).unwrap();
        // Simulate Claude Code appending a line between the read that produced `stale_original`
        // and the delete: `rewrite_keeping_from` is called with that now-stale snapshot directly,
        // the same shape `rewrite_keeping`'s own two internal reads would hit if a real writer
        // landed in the gap between them (too small a window for a test thread to land in reliably).
        std::fs::write(
            &p,
            format!(
                "{}\n{}",
                String::from_utf8(stale_original.clone()).unwrap(),
                line("c", 3000, "/x")
            ),
        )
        .unwrap();
        let err = rewrite_keeping_from(&p, &stale_original, |candidate, _ts_ms| candidate != id).unwrap_err();
        assert!(matches!(err, AppError::Busy { .. }));
    }

    #[test]
    fn clear_older_than_removes_entries_before_the_cutoff() {
        let day_ms = 86_400_000;
        let now = 10 * day_ms;
        let (_d, p) = file(&[line("old", now - 5 * day_ms, "/x"), line("new", now - day_ms, "/x")].join("\n"));
        assert_eq!(clear_older_than(&p, 2, now).unwrap(), WriteOutcome::Written);
        assert_eq!(displays(&p), ["new"]);
    }

    #[test]
    fn clear_older_than_nothing_to_remove_is_unchanged() {
        let now = 10_000_000;
        let (_d, p) = file(&line("new", now, "/x"));
        let before = std::fs::read(&p).unwrap();
        assert_eq!(clear_older_than(&p, 30, now).unwrap(), WriteOutcome::Unchanged);
        assert_eq!(std::fs::read(&p).unwrap(), before);
    }
}
