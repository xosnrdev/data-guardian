# Data Guardian

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust Version](https://img.shields.io/badge/rust-1.78%2B-orange.svg)
![Platform Support](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)
[![CI](https://github.com/xosnrdev/data-guardian/actions/workflows/ci.yml/badge.svg)](https://github.com/xosnrdev/data-guardian/actions/workflows/ci.yml)

Data Guardian is a system service for monitoring and optimizing application data usage. It provides real-time monitoring, alerts, and insights into how applications consume data on your system.

## Features

- **Real-time Monitoring**: Track data usage across all running applications
- **Smart Alerts**: Receive notifications when applications exceed data thresholds
- **Local-First**: All data processing happens locally for maximum privacy
- **Efficient Storage**: Compressed historical data for trend analysis
- **Cross-Platform**: Supports Linux, macOS, and Windows
- **Low Overhead**: Minimal system resource usage
- **Configurable**: Flexible settings for thresholds and intervals

## Installation

### Prerequisites

- Rust 1.78 or higher
- Cargo package manager
- Platform-specific dependencies:
  - Linux: `libdbus-1-dev`
  - macOS: None
  - Windows: None

### From Source

```bash
# Clone the repository
git clone https://github.com/xosnrdev/data-guardian.git
cd data-guardian

# Build and install
cargo install --path .
```

### From Cargo

```bash
cargo install data-guardian
```

## Usage

### Quick Start

1. Start the service:
   ```bash
   dg
   ```

2. The service will run in the background and monitor data usage

3. Receive notifications when applications exceed thresholds

### Configuration

Create or modify `~/.config/DataGuardian/local.toml`:

```toml
# Data limit in bytes before triggering alerts
data_limit = 1073741824  # 1 GB

# How often to check process data usage (in seconds)
check_interval_seconds = 60

# How often to save usage data to disk (in seconds)
persistence_interval_seconds = 300  # 5 minutes
```

### Environment Variables

- `DATAGUARDIAN_DATA_LIMIT`: Override data limit
- `DATAGUARDIAN_CHECK_INTERVAL_SECONDS`: Override check interval
- `DATAGUARDIAN_PERSISTENCE_INTERVAL_SECONDS`: Override persistence interval
- `RUST_LOG`: Set logging level (error, warn, info, debug, trace)

## Development

### Building

```bash
# Build debug version
cargo build

# Build release version
cargo build --release
```

### Testing

```bash
# Run all tests
make test

# Run specific test
cargo test test_name

# Run with logging
cargo test -- --nocapture
```

### Code Quality

```bash
# Run all checks
make all

# Format code
make fmt

# Run lints
make lint
```

## License

Licensed under the [MIT License](LICENSE)

## Acknowledgments

- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) for system information
- [notify-rust](https://github.com/hoodie/notify-rust) for notifications
- [tokio](https://tokio.rs/) for async runtime

## Roadmap

- [ ] Web interface for data visualization
- [ ] Application grouping and categories
- [ ] Network interface filtering
- [ ] Custom alert rules
- [ ] Data usage predictions
- [ ] Export and reporting features

