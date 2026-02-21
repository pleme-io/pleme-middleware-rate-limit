//! General API rate limiter

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tokio::sync::Mutex;
use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    body::Body,
};
use tracing::warn;

use crate::{config::RateLimitConfig, error::RateLimitError};

/// Rate limiter state tracking
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    attempts: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if request should be rate limited
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut attempts = self.attempts.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get or create attempt list for this key
        let attempt_list = attempts.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old attempts outside the window
        let window_start = now.saturating_sub(self.config.rate_window_secs);
        attempt_list.retain(|&timestamp| timestamp > window_start);

        // Check if we've exceeded the limit
        if attempt_list.len() >= self.config.max_requests_per_window as usize {
            warn!("Rate limit exceeded for key: {}", key);
            return Err(RateLimitError::Exceeded(format!(
                "Maximum {} requests per {} seconds exceeded",
                self.config.max_requests_per_window,
                self.config.rate_window_secs
            )));
        }

        // Record this attempt
        attempt_list.push(now);

        Ok(())
    }

    /// Clean up old entries periodically
    pub async fn cleanup(&self) {
        let mut attempts = self.attempts.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now.saturating_sub(self.config.rate_window_secs);

        // Remove entries with no recent attempts
        attempts.retain(|_, timestamps| {
            timestamps.retain(|&t| t > window_start);
            !timestamps.is_empty()
        });
    }
}

/// Rate limiting middleware for Axum
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = addr.ip();
    let path = request.uri().path();

    // Create rate limit key based on IP and path
    let key = format!("{}:{}", ip, path);

    // Check rate limit
    match limiter.check_rate_limit(&key).await {
        Ok(()) => {
            // Request is within limits, proceed
            Ok(next.run(request).await)
        }
        Err(RateLimitError::Exceeded(_)) => {
            warn!("Rate limit exceeded for IP {} on path {}", ip, path);
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        Err(_) => {
            // Other errors, allow request but log
            warn!("Rate limit check failed for IP {} on path {}", ip, path);
            Ok(next.run(request).await)
        }
    }
}
