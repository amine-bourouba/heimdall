//! Fuzzy and exact ("Match Word") matching (Utilities/HighlightedText.swift), with highlight
//! ranges in UTF-16 code units, the unit JavaScript strings index by.
//!
//! Characters are compared one by one after lower-casing each, so every match maps back to a
//! character of the original text. The Swift reference app compared grapheme clusters; the two differ only inside
//! multi-scalar emoji and combining sequences.

use crate::models::prefs::SearchMode;

/// A match's score (higher is better) and its highlight ranges, `[start, end)` in UTF-16 units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub score: i32,
    pub ranges: Vec<(u32, u32)>,
}

/// One character of the text: its lower-case form and where it sits in UTF-16.
struct Char {
    lower: String,
    start16: u32,
    len16: u32,
    raw: char,
}

fn chars(text: &str) -> Vec<Char> {
    let mut at = 0;
    text.chars()
        .map(|c| {
            let len16 = c.len_utf16() as u32;
            let ch = Char {
                lower: c.to_lowercase().collect(),
                start16: at,
                len16,
                raw: c,
            };
            at += len16;
            ch
        })
        .collect()
}

pub fn matches(text: &str, query: &str, mode: SearchMode) -> Option<Match> {
    match mode {
        SearchMode::Fuzzy => fuzzy(text, query),
        SearchMode::Exact => exact(text, query),
    }
}

/// SearchOverlayView.score: the match's score, or 0 when there is none.
pub fn score(text: &str, query: &str, mode: SearchMode) -> i32 {
    matches(text, query, mode).map_or(0, |m| m.score)
}

/// Case-insensitive substring match (HighlightedText.substringMatch). Scored on the first
/// occurrence: 100, +40 at the start, else +20 after one of ` -/._:`, minus one point per 20
/// characters of text (at most 30). Ranges cover every non-overlapping occurrence. An empty query
/// scores 0 with no ranges.
pub fn exact(text: &str, query: &str) -> Option<Match> {
    if query.is_empty() {
        return Some(Match {
            score: 0,
            ranges: Vec::new(),
        });
    }
    let cs = chars(text);
    let q: Vec<String> = query.chars().map(|c| c.to_lowercase().collect()).collect();
    let occurs_at = |i: usize| i + q.len() <= cs.len() && cs[i..i + q.len()].iter().zip(&q).all(|(c, q)| c.lower == *q);
    let mut ranges = Vec::new();
    let mut first = None;
    let mut i = 0;
    while i < cs.len() {
        if occurs_at(i) {
            first.get_or_insert(i);
            let last = &cs[i + q.len() - 1];
            ranges.push((cs[i].start16, last.start16 + last.len16));
            i += q.len();
        } else {
            i += 1;
        }
    }
    let first = first?;
    let mut score = 100;
    if first == 0 {
        score += 40;
    } else if [' ', '-', '/', '.', '_', ':'].contains(&cs[first - 1].raw) {
        score += 20;
    }
    let length: usize = cs.iter().map(|c| c.lower.chars().count()).sum();
    score -= (length / 20).min(30) as i32;
    Some(Match { score, ranges })
}

/// Subsequence match (HighlightedText.fuzzyMatch): +10 per matched character, +8 at the start of
/// the text, else +5 when right after the previous match, else +3 after one of ` -/._`; minus the
/// gap since the previous match. Matches scoring under 5 points per query character are rejected.
/// A one-character query must appear as is and scores 10. Ranges are one per matched character.
pub fn fuzzy(text: &str, query: &str) -> Option<Match> {
    let q: Vec<String> = query.chars().map(|c| c.to_lowercase().collect()).collect();
    if q.is_empty() {
        return Some(Match {
            score: 0,
            ranges: Vec::new(),
        });
    }
    let cs = chars(text);
    if q.len() < 2 {
        let c = cs.iter().find(|c| c.lower == q[0])?;
        return Some(Match {
            score: 10,
            ranges: vec![(c.start16, c.start16 + c.len16)],
        });
    }
    let mut score: i32 = 0;
    let mut ranges = Vec::new();
    let mut prev: Option<usize> = None;
    let mut next = 0;
    for (i, c) in cs.iter().enumerate() {
        if next == q.len() {
            break;
        }
        if c.lower != q[next] {
            continue;
        }
        score += 10;
        if i == 0 {
            score += 8;
        } else if prev == Some(i - 1) {
            score += 5;
        } else if [' ', '-', '/', '.', '_'].contains(&cs[i - 1].raw) {
            score += 3;
        }
        if let Some(p) = prev {
            score -= (i - p - 1) as i32;
        }
        ranges.push((c.start16, c.start16 + c.len16));
        prev = Some(i);
        next += 1;
    }
    (next == q.len() && score >= 5 * q.len() as i32).then_some(Match { score, ranges })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from PoirotTests/Utilities/HighlightedTextTests.swift (matching assertions).
    #[test]
    fn matches_case_insensitive_substring() {
        assert!(exact("Fix SSH tunnel", "ssh").is_some());
        assert!(exact("fix ssh", "SSH").is_some());
    }
    #[test]
    fn rejects_scattered_subsequence_that_fuzzy_would_match() {
        assert!(fuzzy("swordfish", "ssh").is_some());
        assert!(exact("swordfish", "ssh").is_none());
    }
    #[test]
    fn ranks_prefix_over_boundary_over_mid() {
        let s = |t: &str| exact(t, "ssh").map_or(0, |m| m.score);
        assert!(s("ssh config") > s("fix ssh now"));
        assert!(s("fix ssh now") > s("misshapen"));
    }
    #[test]
    fn empty_query_returns_zero() {
        assert_eq!(exact("anything", "").unwrap().score, 0);
    }

    // The fuzzyMatch cases of PoirotTests/Models/PlanTests.swift and MemoryFileTests.swift (M1-10).
    #[test]
    fn fuzzy_match_matches_plan_name_and_content() {
        assert!(fuzzy("Abstract Wobbling Mccarthy", "wobbling").is_some_and(|m| m.score > 0));
        assert!(fuzzy("This plan describes the feature implementation steps", "feature").is_some_and(|m| m.score > 0));
        assert!(fuzzy("Abstract Wobbling", "zzzzz").is_none());
    }
    #[test]
    fn fuzzy_match_matches_memory_name_and_content() {
        assert!(fuzzy("Debugging", "debug").is_some_and(|m| m.score > 0));
        assert!(fuzzy("Always use bun instead of npm for this project", "bun").is_some_and(|m| m.score > 0));
        assert!(fuzzy("Debugging", "zzzzz").is_none());
    }

    // Worked examples from docs/spec/search-overlay.md §2.2-2.4.
    #[test]
    fn spec_examples_score_and_highlight_as_documented() {
        assert_eq!(
            fuzzy("swordfish", "ssh"),
            Some(Match {
                score: 37,
                ranges: vec![(0, 1), (7, 8), (8, 9)]
            })
        );
        assert_eq!(exact("ssh config", "ssh").unwrap().ranges, [(0, 3)]);
        assert_eq!(fuzzy("ssh config", "ssh").unwrap().ranges, [(0, 1), (1, 2), (2, 3)]);
        assert_eq!(
            fuzzy("xyz", "x"),
            Some(Match {
                score: 10,
                ranges: vec![(0, 1)]
            })
        );
    }
    #[test]
    fn exact_highlights_every_occurrence() {
        assert_eq!(exact("ssh to ssh", "SSH").unwrap().ranges, [(0, 3), (7, 10)]);
    }
    #[test]
    fn a_weak_scattered_match_is_rejected() {
        // Every letter of "abc" is present, but far apart: 30 - gaps < 15.
        assert!(fuzzy("a---------------b---------------c", "abc").is_none());
    }

    #[test]
    fn ranges_are_utf16_offsets_through_emoji_and_cjk() {
        let text = "日本語 🎉 SSH tunnel";
        // 日本語 = 3 units, space = 1, 🎉 = 2 (a surrogate pair), space = 1: "SSH" starts at 7.
        assert_eq!(exact(text, "ssh").unwrap().ranges, [(7, 10)]);
        assert_eq!(fuzzy(text, "ssh").unwrap().ranges, [(7, 8), (8, 9), (9, 10)]);
        assert_eq!(exact(text, "🎉").unwrap().ranges, [(4, 6)]);
        assert_eq!(exact(text, "本語").unwrap().ranges, [(1, 3)]);
        let js_len: u32 = text.encode_utf16().count() as u32;
        assert!(
            fuzzy(text, "tunnel")
                .unwrap()
                .ranges
                .iter()
                .all(|&(_, end)| end <= js_len)
        );
    }

    #[test]
    fn mode_dispatch_and_zero_score_for_no_match() {
        assert_eq!(score("swordfish", "ssh", SearchMode::Exact), 0);
        assert_eq!(score("swordfish", "ssh", SearchMode::Fuzzy), 37);
    }
}
