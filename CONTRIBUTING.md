# Contribution Guide

## Development Setup

1. Install Rust toolchain
2. Clone repository
3. `cargo test` to verify setup

## Workflow

1. Create feature branch
2. Write tests for new functionality
3. Implement code
4. Run `make lint fmt test` before committing
5. Open pull request

## Code Style

- Follow Rustfmt rules
- Clippy must pass with no warnings
- Document public APIs