use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::time::error::Elapsed;

use crate::error::CrawlerError;

static RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();

pub fn get_rate_limiter() -> &'static RateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::new(100, 50))
}

#[derive(Debug)]
pub struct RateLimiter {
    tokens: AtomicUsize,
    last_update: AtomicUsize,
    capacity: usize,
    refill_rate: usize,
}

impl RateLimiter {
    pub fn new(capacity: usize, refill_rate: usize) -> Self {
        Self {
            tokens: AtomicUsize::new(capacity),
            last_update: AtomicUsize::new(0),
            capacity,
            refill_rate,
        }
    }
    pub fn check_tokens(&self) -> Result<(), CrawlerError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let last = self.last_update.load(Ordering::Relaxed);

        let elapsed = now.saturating_sub(last);
        let add_tokens = (elapsed.saturating_mul(self.refill_rate)).min(self.capacity);
        self.tokens.fetch_add(add_tokens, Ordering::Relaxed);
        self.tokens.fetch_min(self.capacity, Ordering::Relaxed);
        if self.tokens.load(Ordering::Relaxed) > 0 {
            self.last_update.store(now, Ordering::Relaxed);
            self.consume_token();
            Ok(())
        } else {
            let wait_time = (self.refill_rate - elapsed) % self.refill_rate;
            Err(CrawlerError::RateLimitError(wait_time as u64))
        }
    }
    pub fn consume_token(&self) {
        self.tokens.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_initial_tokens() {
        let limiter = RateLimiter::new(2, 1); // Small values to test quickly
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err());
    }

    #[test]
    fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(2, 1);
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err());

        thread::sleep(Duration::from_secs(1)); // Wait for one token refill
        assert!(limiter.check_tokens().is_ok());

        thread::sleep(Duration::from_secs(1)); // Wait for another token refill
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err()); // Should be empty again
    }

    #[test]
    fn test_rate_limiter_partial_refill() {
        let limiter = RateLimiter::new(3, 1);
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err()); // Out of tokens

        thread::sleep(Duration::from_secs(2)); // Should refill 2 tokens
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err()); // Should be empty again
    }

    #[test]
    fn test_rate_limiter_does_not_exceed_capacity() {
        let limiter = RateLimiter::new(5, 2);
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err()); // Out of tokens

        thread::sleep(Duration::from_secs(5)); // Should refill, but not exceed capacity
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_ok());
        assert!(limiter.check_tokens().is_err()); // Still should not exceed capacity
    }
}
