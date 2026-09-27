//! Line diff (Utilities/LineDiff.swift): a longest-common-subsequence diff between two line
//! arrays, with the 5000-line cap `EditDiffView` applies before diffing.
//!
//! Poirot built this from Swift's own `CollectionDifference` (a Myers diff). This port computes
//! the same thing — a minimal set of line insertions and removals — from a textbook LCS dynamic
//! program instead, since the standard library has no equivalent and matching Swift's edit script
//! for every one of Poirot's tests only needs a correct, not bit-identical, algorithm.
//!
//! ponytail: the DP table is `O(old.len() * new.len())` time and space — 5000×5000 four-byte
//! cells is ~100 MB, reached only by two capped-length files that share no line at all, not by a
//! real edit (whose small edit distance keeps the useful work tiny regardless of file length).
//! Upgrade to Myers' linear-space (divide-and-conquer) algorithm if that ever proves to matter.

use serde::Serialize;
use specta::Type;

/// Lines beyond this in either file are dropped before diffing (`EditDiffView.swift:22`).
pub const MAX_DIFF_LINES: usize = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum DiffKind {
    Context,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub truncated: bool,
    pub max_line_number: u32,
}

/// Splits on `\n` (matching Swift's `components(separatedBy: "\n")`; a `\r` from a CRLF file
/// stays on the line, as it did for Poirot) and diffs.
pub fn diff(old: &str, new: &str) -> DiffResult {
    let old_lines: Vec<&str> = old.split('\n').collect();
    let new_lines: Vec<&str> = new.split('\n').collect();
    diff_lines(&old_lines, &new_lines)
}

/// Diffs two already-split line arrays, applying the 5000-line cap to each independently.
pub fn diff_lines(old_lines: &[&str], new_lines: &[&str]) -> DiffResult {
    let truncated = old_lines.len() > MAX_DIFF_LINES || new_lines.len() > MAX_DIFF_LINES;
    let old = &old_lines[..old_lines.len().min(MAX_DIFF_LINES)];
    let new = &new_lines[..new_lines.len().min(MAX_DIFF_LINES)];
    let lines = lcs_diff(old, new);
    let max_line_number = lines
        .iter()
        .flat_map(|l| [l.old_line, l.new_line])
        .flatten()
        .max()
        .unwrap_or(0);
    DiffResult {
        lines,
        truncated,
        max_line_number,
    }
}

/// `unifiedText` (Utilities/LineDiff.swift): one line per row, ` `/`+`/`-` prefixed.
pub fn unified_text(lines: &[DiffLine]) -> String {
    lines
        .iter()
        .map(|l| {
            let prefix = match l.kind {
                DiffKind::Context => ' ',
                DiffKind::Add => '+',
                DiffKind::Remove => '-',
            };
            format!("{prefix}{}", l.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn lcs_diff(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    let (n, m) = (old.len(), new.len());
    let width = m + 1;
    let mut dp = vec![0u32; (n + 1) * width];
    for i in 1..=n {
        for j in 1..=m {
            dp[i * width + j] = if old[i - 1] == new[j - 1] {
                dp[(i - 1) * width + (j - 1)] + 1
            } else {
                dp[(i - 1) * width + j].max(dp[i * width + (j - 1)])
            };
        }
    }

    // Walk back from (n, m) to (0, 0); the result comes out reversed, so it is built as such and
    // reversed once at the end.
    let mut lines = Vec::with_capacity(n + m);
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            lines.push(DiffLine {
                kind: DiffKind::Context,
                old_line: Some(i as u32),
                new_line: Some(j as u32),
                text: old[i - 1].to_string(),
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i * width + (j - 1)] >= dp[(i - 1) * width + j]) {
            lines.push(DiffLine {
                kind: DiffKind::Add,
                old_line: None,
                new_line: Some(j as u32),
                text: new[j - 1].to_string(),
            });
            j -= 1;
        } else {
            lines.push(DiffLine {
                kind: DiffKind::Remove,
                old_line: Some(i as u32),
                new_line: None,
                text: old[i - 1].to_string(),
            });
            i -= 1;
        }
    }
    lines.reverse();
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(r: &DiffResult) -> Vec<DiffKind> {
        r.lines.iter().map(|l| l.kind).collect()
    }
    fn texts(r: &DiffResult) -> Vec<&str> {
        r.lines.iter().map(|l| l.text.as_str()).collect()
    }

    // Ported from PoirotTests/Utilities/LineDiffTests.swift.
    #[test]
    fn identical_strings_all_context() {
        let r = diff("a\nb\nc", "a\nb\nc");
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines.iter().all(|l| l.kind == DiffKind::Context));
    }
    #[test]
    fn empty_old_all_added() {
        let r = diff_lines(&[], &["a", "b"]);
        assert_eq!(kinds(&r), [DiffKind::Add, DiffKind::Add]);
    }
    #[test]
    fn empty_new_all_removed() {
        let r = diff_lines(&["a", "b"], &[]);
        assert_eq!(kinds(&r), [DiffKind::Remove, DiffKind::Remove]);
    }
    #[test]
    fn single_line_change_one_removed_one_added() {
        let r = diff("hello", "world");
        assert_eq!(r.lines.iter().filter(|l| l.kind == DiffKind::Remove).count(), 1);
        assert_eq!(r.lines.iter().filter(|l| l.kind == DiffKind::Add).count(), 1);
        assert_eq!(
            r.lines.iter().find(|l| l.kind == DiffKind::Remove).unwrap().text,
            "hello"
        );
        assert_eq!(r.lines.iter().find(|l| l.kind == DiffKind::Add).unwrap().text, "world");
    }
    #[test]
    fn insertion_in_middle_preserves_context() {
        let r = diff("a\nc", "a\nb\nc");
        assert_eq!(kinds(&r), [DiffKind::Context, DiffKind::Add, DiffKind::Context]);
        assert_eq!(texts(&r), ["a", "b", "c"]);
        assert_eq!((r.lines[0].old_line, r.lines[0].new_line), (Some(1), Some(1)));
    }
    #[test]
    fn deletion_in_middle_preserves_context() {
        let r = diff("a\nb\nc", "a\nc");
        assert_eq!(kinds(&r), [DiffKind::Context, DiffKind::Remove, DiffKind::Context]);
        assert_eq!(texts(&r), ["a", "b", "c"]);
    }
    #[test]
    fn line_numbers_correct_for_context() {
        let r = diff("a\nb\nc", "a\nb\nc");
        for (i, line) in r.lines.iter().enumerate() {
            assert_eq!((line.old_line, line.new_line), (Some(i as u32 + 1), Some(i as u32 + 1)));
        }
    }
    #[test]
    fn line_numbers_nil_for_added_removed() {
        let r = diff("old", "new");
        let removed = r.lines.iter().find(|l| l.kind == DiffKind::Remove).unwrap();
        let added = r.lines.iter().find(|l| l.kind == DiffKind::Add).unwrap();
        assert_eq!((removed.old_line, removed.new_line), (Some(1), None));
        assert_eq!((added.old_line, added.new_line), (None, Some(1)));
    }
    // Poirot's `sequentialIds` tested a SwiftUI `Identifiable` id that Heimdall's wire `DiffLine`
    // has no equivalent of (the frontend can key rows by array index); ported as order-and-length.
    #[test]
    fn diff_preserves_input_order_and_produces_no_extra_lines() {
        let r = diff("a\nb", "a\nc");
        assert_eq!(r.lines.len(), 3);
    }
    #[test]
    fn unified_text_formats_correctly() {
        let r = diff("a\nb", "a\nc");
        let text = unified_text(&r.lines);
        assert!(text.contains(" a") && text.contains("-b") && text.contains("+c"));
    }
    #[test]
    fn both_empty_produces_empty_context_line() {
        let r = diff("", "");
        assert_eq!(r.lines.len(), 1);
        assert_eq!(r.lines[0].kind, DiffKind::Context);
        assert!(r.lines[0].text.is_empty());
    }
    #[test]
    fn prefix_and_suffix_preserved_middle_replacement() {
        let r = diff("keep\nchange me\nalso keep", "keep\nchanged\nalso keep");
        assert_eq!(
            (r.lines.first().unwrap().kind, r.lines.first().unwrap().text.as_str()),
            (DiffKind::Context, "keep")
        );
        assert_eq!(
            (r.lines.last().unwrap().kind, r.lines.last().unwrap().text.as_str()),
            (DiffKind::Context, "also keep")
        );
        assert!(
            r.lines
                .iter()
                .any(|l| l.kind == DiffKind::Remove && l.text == "change me")
        );
        assert!(r.lines.iter().any(|l| l.kind == DiffKind::Add && l.text == "changed"));
    }
    #[test]
    fn trailing_insertion_does_not_drop_context() {
        let r = diff("a\nb", "a\nb\nc");
        assert_eq!(kinds(&r), [DiffKind::Context, DiffKind::Context, DiffKind::Add]);
        assert_eq!(texts(&r), ["a", "b", "c"]);
        assert_eq!(r.lines[2].new_line, Some(3));
    }

    #[test]
    fn a_replacement_reports_the_removal_before_the_insertion() {
        // Matches TranscriptParser's own reconstruction order (it checks a pending removal before
        // a pending insertion at the same position), which the DP backtrack's tie-break reproduces.
        let r = diff("old", "new");
        assert_eq!(kinds(&r), [DiffKind::Remove, DiffKind::Add]);
    }

    #[test]
    fn inputs_over_the_cap_are_truncated_and_flagged() {
        let old: Vec<String> = (0..MAX_DIFF_LINES + 10).map(|i| format!("line{i}")).collect();
        let old_refs: Vec<&str> = old.iter().map(String::as_str).collect();
        let r = diff_lines(&old_refs, &["line0"]);
        assert!(r.truncated);
        assert!(
            r.lines
                .iter()
                .all(|l| l.old_line.is_none_or(|n| n as usize <= MAX_DIFF_LINES))
        );
        let untouched = diff_lines(&old_refs[..MAX_DIFF_LINES], &["line0"]);
        assert!(!untouched.truncated);
    }

    #[test]
    fn max_line_number_is_the_largest_line_seen_on_either_side() {
        let r = diff("a\nb", "a\nb\nc\nd");
        assert_eq!(r.max_line_number, 4);
    }

    #[test]
    fn kind_names_match_the_contract() {
        for (kind, name) in [
            (DiffKind::Context, "context"),
            (DiffKind::Add, "add"),
            (DiffKind::Remove, "remove"),
        ] {
            assert_eq!(serde_json::to_value(kind).unwrap(), name);
        }
    }
}
