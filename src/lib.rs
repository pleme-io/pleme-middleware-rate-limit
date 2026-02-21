//! Rate limiting middleware for Axum web services
//!
//! Provides flexible rate limiting for API endpoints to prevent abuse:
//! - General API rate limiting (IP + path based)
//! - Login-specific rate limiting with account lockout
//! - Configurable time windows and limits
//! - Automatic cleanup of old entries
//!
//! # Example
//! ```rust
//! use pleme_middleware_rate_limit::{RateLimiter, RateLimitConfig};
//! use axum::{Router, routing::get};
//!
//! let config = RateLimitConfig::default();
//! let limiter = RateLimiter::new(config);
//!
//! let app = Router::new()
//!     .route("/api/endpoint", get(handler))
//!     .layer(axum::middleware::from_fn_with_state(
//!         limiter.clone(),
//!         pleme_middleware_rate_limit::rate_limit_middleware
//!     ));
//! ```

mod limiter;
mod login;
mod config;
mod error;

pub use limiter::RateLimiter;
pub use login::LoginRateLimiter;
pub use config::RateLimitConfig;
pub use error::RateLimitError;

// Re-export middleware function
pub use limiter::rate_limit_middleware;
