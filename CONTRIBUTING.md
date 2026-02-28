# Contributing

## Development Setup

1. Install Rust 1.73+ via [rustup](https://rustup.rs)
2. Clone the repository
3. Run `cargo test` to verify the build

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Write tests for new functionality
- Keep functions focused and under 50 lines where possible

## Pull Requests

- Fork the repository and create a feature branch from `main`
- Keep commits focused and atomic
- Include tests for any new functionality
- Update documentation if behavior changes
- Ensure CI passes before requesting review

## Testing

```bash
cargo test --all                    # Run all tests
cargo test --lib                    # Unit tests only
cargo test --test integration_test  # Integration tests
cargo bench                         # Run benchmarks
```

## SDK Development

```bash
cd sdk
npm ci
npm test
npm run build
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for design decisions and data flow.
