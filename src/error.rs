//! Rate limiting errors

/// Rate limiting error types
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {0}")]
    Exceeded(String),

    #[error("Account locked until {0}")]
    AccountLocked(u64),
}
