//! Token-bucket pacing a tile source declares for its own request budget.
//!
//! A tile service that publishes a rate limit can carry it on the source
//! declaration. The provider builds one [`everymap_core::client::TokenBucket`]
//! from the declaration and installs it on the shared client, so every fetch
//! is paced by the same pipeline that handles retries, timeouts and error
//! mapping. A source that declares no pacing is never serialized.

use everymap_core::error::{EveryMapError, EveryMapResult};
use serde::{Deserialize, Serialize};

/// Immediate requests a source admits when it asks for the documented default
/// pacing.
pub const DEFAULT_CAPACITY_TOKENS: u32 = 8;

/// Sustained request rate, in tokens per second, a source admits when it asks
/// for the documented default pacing.
pub const DEFAULT_REFILL_TOKENS_PER_SECOND: f64 = 8.0;

/// A tile source's declared request budget.
///
/// The bucket starts full with [`Self::capacity_tokens`] tokens and refills
/// continuously at [`Self::refill_tokens_per_second`]. Each fetch consumes one
/// token; a fetch that finds the bucket empty waits for the next refill.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TileSourcePacing {
    /// Immediate requests the bucket admits before pacing starts.
    pub capacity_tokens: u32,
    /// Sustained request rate in tokens per second.
    pub refill_tokens_per_second: f64,
}

impl TileSourcePacing {
    /// Builds one declaration from its two rates.
    ///
    /// The values are not checked here; a source validates them with
    /// [`Self::validate`] while it knows its own name for the diagnostic.
    ///
    /// # Parameters
    /// - `capacity_tokens` — immediate requests the bucket admits.
    /// - `refill_tokens_per_second` — sustained request rate.
    pub const fn new(capacity_tokens: u32, refill_tokens_per_second: f64) -> Self {
        Self {
            capacity_tokens,
            refill_tokens_per_second,
        }
    }

    /// Refuses a rate the shared token bucket cannot enforce.
    ///
    /// # Parameters
    /// - `source_name` — the declaring source's name, used in the diagnostic.
    ///
    /// # Returns
    /// `Ok(())` when the bucket can admit and refill requests.
    ///
    /// # Errors
    /// Returns [`EveryMapError::ValidationError`] when the capacity is zero,
    /// or when the refill rate is not finite or not strictly positive.
    pub fn validate(&self, source_name: &str) -> EveryMapResult<()> {
        if self.capacity_tokens == 0 {
            return Err(EveryMapError::ValidationError(format!(
                "the {source_name} tile source declares a token-bucket capacity of 0 tokens, \
                 which must be at least 1"
            )));
        }
        if !self.refill_tokens_per_second.is_finite() || self.refill_tokens_per_second <= 0.0 {
            return Err(EveryMapError::ValidationError(format!(
                "the {source_name} tile source declares a refill rate of {} tokens per second, \
                 which must be finite and positive",
                self.refill_tokens_per_second
            )));
        }
        Ok(())
    }
}

impl Default for TileSourcePacing {
    /// The documented default budget: an eight-request burst that refills at
    /// eight requests per second.
    fn default() -> Self {
        Self {
            capacity_tokens: DEFAULT_CAPACITY_TOKENS,
            refill_tokens_per_second: DEFAULT_REFILL_TOKENS_PER_SECOND,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TileSourcePacing, DEFAULT_CAPACITY_TOKENS, DEFAULT_REFILL_TOKENS_PER_SECOND};
    use everymap_core::error::EveryMapError;

    #[test]
    fn the_default_budget_is_the_documented_eight_by_eight() {
        let pacing = TileSourcePacing::default();
        assert_eq!(pacing.capacity_tokens, DEFAULT_CAPACITY_TOKENS);
        assert_eq!(
            pacing.refill_tokens_per_second,
            DEFAULT_REFILL_TOKENS_PER_SECOND
        );
        assert!(pacing.validate("test/source").is_ok());
    }

    #[test]
    fn a_zero_capacity_is_refused_and_names_the_source() {
        let error = TileSourcePacing::new(0, 1.0)
            .validate("test/source")
            .unwrap_err();
        match error {
            EveryMapError::ValidationError(detail) => {
                assert!(detail.contains("test/source"), "{detail}");
            }
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn a_non_positive_or_non_finite_refill_rate_is_refused() {
        for refill_tokens_per_second in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(TileSourcePacing::new(8, refill_tokens_per_second)
                .validate("test/source")
                .is_err());
        }
    }
}
