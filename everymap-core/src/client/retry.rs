//! Retry policy for idempotent requests.

use std::time::Duration;

/// Default number of attempts, including the first one.
const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Default delay before the second attempt.
const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Default cap for exponential backoff.
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(2);

/// Exponential backoff parameters for idempotent requests.
///
/// Only methods for which [`crate::http::Method::is_idempotent`] returns `true`
/// are retried. The shared client retries transport failures, `5xx` responses,
/// and short `429` responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Total attempts allowed, including the first. Clamped to at least one.
    pub max_attempts: u32,
    /// Delay before the second attempt; doubled for each later attempt.
    pub base_delay: Duration,
    /// Upper bound for the exponential delay.
    pub max_delay: Duration,
}

impl RetryPolicy {
    /// A policy that sends every request exactly once.
    pub fn disabled() -> Self {
        Self {
            max_attempts: 1,
            ..Self::default()
        }
    }

    /// Whether a second attempt is allowed.
    pub fn is_enabled(&self) -> bool {
        self.max_attempts > 1
    }

    /// Delay before the attempt that follows `completed_attempt`.
    ///
    /// `completed_attempt` is one-based; the first retry waits `base_delay`,
    /// the second twice that, capped at `max_delay`.
    pub fn delay_after_attempt(&self, completed_attempt: u32) -> Duration {
        let exponent = completed_attempt.saturating_sub(1).min(16);
        let multiplier = 1_u32 << exponent;
        self.base_delay
            .saturating_mul(multiplier)
            .min(self.max_delay)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay: DEFAULT_BASE_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_retries_twice() {
        let policy = RetryPolicy::default();
        assert!(policy.is_enabled());
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.delay_after_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_after_attempt(2), Duration::from_millis(200));
        assert_eq!(policy.delay_after_attempt(3), Duration::from_millis(400));
        assert_eq!(policy.delay_after_attempt(10), policy.max_delay);
    }

    #[test]
    fn disabled_policy_sends_once() {
        let policy = RetryPolicy::disabled();
        assert!(!policy.is_enabled());
        assert_eq!(policy.max_attempts, 1);
    }

    #[test]
    fn delay_is_capped_and_overflow_safe() {
        let policy = RetryPolicy {
            max_attempts: 100,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
        };
        assert_eq!(policy.delay_after_attempt(u32::MAX), Duration::from_secs(5));
    }
}
