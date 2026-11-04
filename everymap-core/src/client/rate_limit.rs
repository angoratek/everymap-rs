//! Token-bucket rate limiting for outgoing provider requests.

use crate::client::sleep;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// One token is the cost of a single request.
const REQUEST_TOKEN_COST: f64 = 1.0;

struct BucketState {
    tokens: f64,
    last_refill: Instant,
}

/// A token bucket that paces outgoing requests.
///
/// The bucket starts full and refills continuously at
/// [`TokenBucket::refill_per_second`] up to [`TokenBucket::capacity`]. Each
/// request removes one token; when none is available, [`TokenBucket::acquire`]
/// waits until the next token refills.
pub struct TokenBucket {
    capacity: f64,
    refill_per_second: f64,
    state: Mutex<BucketState>,
}

impl TokenBucket {
    /// Create a full bucket with the given burst capacity and refill rate.
    ///
    /// # Panics
    ///
    /// Panics when `capacity` is zero or `refill_per_second` is not positive,
    /// because such a bucket could never admit a request.
    pub fn new(capacity: u32, refill_per_second: f64) -> Self {
        assert!(capacity > 0, "token bucket capacity must be positive");
        assert!(
            refill_per_second.is_finite() && refill_per_second > 0.0,
            "token bucket refill rate must be finite and positive"
        );
        let capacity = f64::from(capacity);
        Self {
            capacity,
            refill_per_second,
            state: Mutex::new(BucketState {
                tokens: capacity,
                last_refill: Instant::now(),
            }),
        }
    }

    /// Maximum number of tokens the bucket can hold.
    pub fn capacity(&self) -> f64 {
        self.capacity
    }

    /// Tokens added per second.
    pub fn refill_per_second(&self) -> f64 {
        self.refill_per_second
    }

    /// Wait until one token is available and consume it.
    pub async fn acquire(&self) {
        loop {
            match self.consume().await {
                None => return,
                Some(wait) => sleep::sleep(wait).await,
            }
        }
    }

    /// Try to consume one token, returning how long to wait when none is free.
    async fn consume(&self) -> Option<Duration> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(state.last_refill);
        state.last_refill = now;
        state.tokens =
            (state.tokens + elapsed.as_secs_f64() * self.refill_per_second).min(self.capacity);

        if state.tokens >= REQUEST_TOKEN_COST {
            state.tokens -= REQUEST_TOKEN_COST;
            None
        } else {
            let missing_tokens = REQUEST_TOKEN_COST - state.tokens;
            Some(Duration::from_secs_f64(
                missing_tokens / self.refill_per_second,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn burst_capacity_is_admitted_without_waiting() {
        let bucket = TokenBucket::new(3, 1.0);
        let start = Instant::now();
        for _ in 0..3 {
            bucket.acquire().await;
        }
        assert!(start.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn request_waits_for_a_refill_when_the_bucket_is_empty() {
        let bucket = Arc::new(TokenBucket::new(1, 1000.0));
        bucket.acquire().await;
        let start = Instant::now();
        bucket.acquire().await;
        assert!(start.elapsed() >= Duration::from_millis(1));
    }

    #[tokio::test]
    async fn concurrent_acquires_are_serialized() {
        let bucket = Arc::new(TokenBucket::new(2, 1000.0));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let bucket = Arc::clone(&bucket);
            handles.push(tokio::spawn(async move { bucket.acquire().await }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        assert_eq!(bucket.capacity(), 2.0);
        assert_eq!(bucket.refill_per_second(), 1000.0);
    }
}
