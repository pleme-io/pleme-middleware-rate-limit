# pleme-middleware-rate-limit

Rate limiting middleware for Axum web services

## Installation

```toml
[dependencies]
pleme-middleware-rate-limit = "0.1"
```

## Usage

```rust
use pleme_middleware_rate_limit::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::new(RateLimitConfig {
    requests_per_second: 10,
    burst_size: 20,
});

let app = Router::new()
    .route("/api", get(handler))
    .layer(limiter.layer());
```

## Development

This project uses [Nix](https://nixos.org/) for reproducible builds:

```bash
nix develop            # Dev shell with Rust toolchain
nix run .#check-all    # cargo fmt + clippy + test
nix run .#publish      # Publish to crates.io (--dry-run supported)
nix run .#regenerate   # Regenerate Cargo.nix
```

## License

MIT - see [LICENSE](LICENSE) for details.
