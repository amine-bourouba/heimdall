//! Context-window usage derived from a transcript's per-turn usage (Models/ContextUsage.swift).

use serde::Serialize;
use specta::Type;

use super::message::Message;

/// The standard Claude context window; the 1M tier is a beta only some models and turns use.
pub const STANDARD_WINDOW: u64 = 200_000;
pub const EXTENDED_WINDOW: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContextUsage {
    /// Context size of the most recent turn that carried usage.
    pub current: u64,
    /// Largest context size seen.
    pub peak: u64,
    pub window: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    /// Context size per turn, in order, for the growth sparkline.
    pub series: Vec<u64>,
}

impl ContextUsage {
    pub fn from_messages(messages: &[Message]) -> Self {
        let usages: Vec<_> = messages.iter().filter_map(|m| m.usage).collect();
        let series: Vec<u64> = usages.iter().map(|u| u.context_tokens()).filter(|&c| c > 0).collect();
        let peak = series.iter().copied().max().unwrap_or(0);
        // A turn above the standard tier must have run on the extended window.
        let window = if peak > STANDARD_WINDOW { EXTENDED_WINDOW } else { STANDARD_WINDOW };
        Self {
            current: series.last().copied().unwrap_or(0),
            peak,
            window,
            input_tokens: usages.iter().map(|u| u.input).sum(),
            output_tokens: usages.iter().map(|u| u.output).sum(),
            cache_read_tokens: usages.iter().map(|u| u.cache_read).sum(),
            cache_creation_tokens: usages.iter().map(|u| u.cache_creation).sum(),
            series,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// How full the window is now (0..=1).
    pub fn fraction(&self) -> f64 {
        if self.window > 0 { (self.current as f64 / self.window as f64).clamp(0.0, 1.0) } else { 0.0 }
    }

    pub fn percent(&self) -> u32 {
        (self.fraction() * 100.0).round() as u32
    }

    /// Fraction (0..=1) of all prompt tokens served from cache across the conversation.
    pub fn cache_hit_rate(&self) -> f64 {
        let prompt = self.input_tokens + self.cache_read_tokens + self.cache_creation_tokens;
        if prompt > 0 { self.cache_read_tokens as f64 / prompt as f64 } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::message::{Role, test_support::message};
    use crate::models::token_usage::TokenUsage;

    fn turn(usage: Option<TokenUsage>) -> Message {
        let mut m = message("m", Role::Assistant, vec![]);
        m.usage = usage;
        m
    }

    // Ported from PoirotTests/Models/ContextUsageTests.swift.
    #[test]
    fn from_computes_current_peak_and_series() {
        let ctx = ContextUsage::from_messages(&[
            turn(Some(TokenUsage::with_cache(10, 5, 1000, 200))),
            turn(Some(TokenUsage::with_cache(20, 8, 5000, 300))),
            turn(None),
            turn(Some(TokenUsage::with_cache(5, 2, 3000, 100))),
        ]);
        assert_eq!(ctx.series, [1210, 5320, 3105]);
        assert_eq!((ctx.current, ctx.peak, ctx.window), (3105, 5320, 200_000));
    }
    #[test]
    fn from_infers_extended_window_above_200k() {
        let ctx = ContextUsage::from_messages(&[turn(Some(TokenUsage::with_cache(2, 213, 437_861, 2414)))]);
        assert_eq!((ctx.current, ctx.window, ctx.percent()), (440_277, 1_000_000, 44));
    }
    #[test]
    fn cache_hit_rate_aggregates_across_turns() {
        let ctx = ContextUsage::from_messages(&[
            turn(Some(TokenUsage::with_cache(100, 0, 300, 100))),
            turn(Some(TokenUsage::with_cache(0, 0, 500, 0))),
        ]);
        assert_eq!(ctx.cache_hit_rate(), 0.8);
    }
    #[test]
    fn from_no_usage_is_empty() {
        let ctx = ContextUsage::from_messages(&[turn(None)]);
        assert!(ctx.is_empty());
        assert_eq!((ctx.current, ctx.fraction()), (0, 0.0));
    }
}
