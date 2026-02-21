//! Rate limiting configuration

use serde::{Deserialize, Serialize};

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable/disable rate limiting
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum requests per window (for general API rate limiting)
    #[serde(default = "default_max_requests")]
    pub max_requests_per_window: u32,

    /// Time window in seconds (for general API rate limiting)
    #[serde(default = "default_rate_window")]
    pub rate_window_secs: u64,

    /// Maximum login attempts before lockout
    #[serde(default = "default_max_login_attempts")]
    pub max_login_attempts: u32,

    /// Account lockout duration in seconds
    #[serde(default = "default_lockout_duration")]
    pub lockout_duration_secs: u64,
}

fn default_enabled() -> bool { true }
fn default_max_requests() -> u32 { 100 }
fn default_rate_window() -> u64 { 60 }
fn default_max_login_attempts() -> u32 { 5 }
fn default_lockout_duration() -> u64 { 300 }

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_window: 100,
            rate_window_secs: 60,
            max_login_attempts: 5,
            lockout_duration_secs: 300,
        }
    }
}
