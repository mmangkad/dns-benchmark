# Contributing to DNS Benchmark

Thanks for your interest in contributing!

## Getting Started

### Prerequisites

- Rust toolchain 1.92.0 via [rustup](https://rustup.rs/) (pinned by rust-toolchain.toml)
- Docker (optional, for container builds)

### Building

```bash
cargo build
```

### Running

```bash
cargo run -- [OPTIONS]
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
```

## Project Structure

```
src/
├── benchmark/     # Benchmark engine and results
├── dns/           # DNS server definitions
├── output/        # Output formatters
├── platform/      # Platform-specific detection
├── cli.rs         # CLI definitions
├── config.rs      # Configuration
├── error.rs       # Error types
├── lib.rs         # Library root
└── main.rs        # Entry point

docker/            # Docker files
examples/          # Example files
tests/             # Test assets
```

## Guidelines

- Keep PRs focused and small
- Add tests for new features
- Update documentation as needed
- Follow existing code style

## License

By contributing, you agree that your contributions will be licensed under the MIT and Apache-2.0 licenses.
