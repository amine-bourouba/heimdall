//! Token usage for one turn (Models/Message.swift `TokenUsage`).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_creation: u64,
}

impl TokenUsage {
    pub const fn new(input: u64, output: u64) -> Self {
        Self {
            input,
            output,
            cache_read: 0,
            cache_creation: 0,
        }
    }

    pub const fn with_cache(input: u64, output: u64, cache_read: u64, cache_creation: u64) -> Self {
        Self {
            input,
            output,
            cache_read,
            cache_creation,
        }
    }

    /// Billable volume for the turn: prompt input plus generated output (the session total sums this).
    pub const fn total(&self) -> u64 {
        self.input + self.output
    }

    /// Prompt tokens the model held as context: fresh input plus cache reads and cache writes.
    pub const fn context_tokens(&self) -> u64 {
        self.input + self.cache_read + self.cache_creation
    }

    /// Fraction (0..=1) of the prompt served from cache.
    pub fn cache_hit_rate(&self) -> f64 {
        let ctx = self.context_tokens();
        if ctx > 0 {
            self.cache_read as f64 / ctx as f64
        } else {
            0.0
        }
    }

    /// `499`, `1.0k`, `125.0k` (TokenUsage.formatted).
    pub fn formatted(&self) -> String {
        let total = self.total();
        if total >= 1000 {
            format!("{:.1}k", total as f64 / 1000.0)
        } else {
            total.to_string()
        }
    }

    /// Component-wise sum, used to total a session.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            input: self.input + other.input,
            output: self.output + other.output,
            cache_read: self.cache_read + other.cache_read,
            cache_creation: self.cache_creation + other.cache_creation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from PoirotTests/Models/TokenUsageTests.swift.
    #[test]
    fn total_sums_input_and_output() {
        assert_eq!(TokenUsage::new(100, 200).total(), 300);
    }
    #[test]
    fn formatted_below_1000_returns_plain_int() {
        assert_eq!(TokenUsage::new(400, 99).formatted(), "499");
    }
    #[test]
    fn formatted_exactly_1000_returns_1_0k() {
        assert_eq!(TokenUsage::new(500, 500).formatted(), "1.0k");
    }
    #[test]
    fn formatted_above_1000_returns_decimal_k() {
        assert_eq!(TokenUsage::new(1000, 500).formatted(), "1.5k");
    }
    #[test]
    fn formatted_zero_tokens_returns_zero() {
        assert_eq!(TokenUsage::new(0, 0).formatted(), "0");
    }
    #[test]
    fn formatted_large_value_correct_format() {
        assert_eq!(TokenUsage::new(50_000, 75_000).formatted(), "125.0k");
    }
    #[test]
    fn context_tokens_sums_prompt_side() {
        assert_eq!(TokenUsage::with_cache(2, 213, 437_861, 2414).context_tokens(), 440_277);
    }
    #[test]
    fn cache_hit_rate_fraction_of_prompt_from_cache() {
        assert_eq!(TokenUsage::with_cache(100, 50, 300, 100).cache_hit_rate(), 0.6);
    }
    #[test]
    fn cache_hit_rate_no_prompt_is_zero() {
        assert_eq!(TokenUsage::new(0, 0).cache_hit_rate(), 0.0);
    }
    #[test]
    fn legacy_init_defaults_cache_to_zero() {
        let usage = TokenUsage::new(100, 200);
        assert_eq!(
            (usage.cache_read, usage.cache_creation, usage.context_tokens()),
            (0, 0, 100)
        );
    }
}
