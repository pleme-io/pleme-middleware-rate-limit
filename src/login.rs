//! Login-specific rate limiter with account lockout

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{config::RateLimitConfig, error::RateLimitError};

/// Login-specific rate limiter with account lockout
#[derive(Clone)]
pub struct LoginRateLimiter {
    config: RateLimitConfig,
    login_attempts: Arc<Mutex<HashMap<String, LoginAttemptInfo>>>,
}

#[derive(Debug)]
struct LoginAttemptInfo {
    attempts: Vec<u64>,
    locked_until: Option<u64>,
}

impl LoginRateLimiter {
    /// Create new login rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check login attempt for user
    pub async fn check_login_attempt(&self, identifier: &str) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut attempts = self.login_attempts.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = attempts.entry(identifier.to_string())
            .or_insert(LoginAttemptInfo {
                attempts: Vec::new(),
                locked_until: None,
            });

        // Check if account is locked
        if let Some(locked_until) = info.locked_until {
            if now < locked_until {
                let remaining = locked_until - now;
                warn!("Login attempt for locked account: {} ({} seconds remaining)",
                    identifier, remaining);
                return Err(RateLimitError::AccountLocked(locked_until));
            } else {
                // Lockout expired, clear it
                info.locked_until = None;
                info.attempts.clear();
            }
        }

        // Remove old attempts
        let window_start = now.saturating_sub(self.config.rate_window_secs);
        info.attempts.retain(|&t| t > window_start);

        // Check if we should lock the account
        if info.attempts.len() >= self.config.max_login_attempts as usize {
            info.locked_until = Some(now + self.config.lockout_duration_secs);
            warn!("Account locked due to too many attempts: {}", identifier);
            return Err(RateLimitError::AccountLocked(info.locked_until.unwrap()));
        }

        Ok(())
    }

    /// Record failed login attempt
    pub async fn record_failed_attempt(&self, identifier: &str) {
        let mut attempts = self.login_attempts.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = attempts.entry(identifier.to_string())
            .or_insert(LoginAttemptInfo {
                attempts: Vec::new(),
                locked_until: None,
            });

        info.attempts.push(now);
        info!("Failed login attempt recorded for: {}", identifier);
    }

    /// Clear attempts after successful login
    pub async fn clear_attempts(&self, identifier: &str) {
        let mut attempts = self.login_attempts.lock().await;
        attempts.remove(identifier);
        info!("Login attempts cleared for: {}", identifier);
    }

    /// Clean up old entries periodically
    pub async fn cleanup(&self) {
        let mut attempts = self.login_attempts.lock().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now.saturating_sub(self.config.rate_window_secs);

        attempts.retain(|_, info| {
            // Keep if locked
            if let Some(locked_until) = info.locked_until {
                if now < locked_until {
                    return true;
                }
            }

            // Remove old attempts
            info.attempts.retain(|&t| t > window_start);
            !info.attempts.is_empty()
        });
    }
}
