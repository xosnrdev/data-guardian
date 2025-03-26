# Data Guardian

[![CI](https://github.com/xosnrdev/data-guardian/actions/workflows/ci.yml/badge.svg)](https://github.com/xosnrdev/data-guardian/actions/workflows/ci.yml)
![Rust Version](https://img.shields.io/badge/rust-1.85%2B-orange.svg)
![Platform Support](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Data Guardian is a system utility that monitors the disk I/O usage of applications running on your computer. It tracks read/write operations and alerts you when applications exceed a configured data threshold.

## Features

- [x] **Process Monitoring**: Tracks disk read/write operations across all running processes
- [x] **Threshold Alerts**: Receive notifications when applications exceed data thresholds
- [x] **Local-First**: All data processing happens locally for maximum privacy
- [x] **Efficient Storage**: Compressed historical data with configurable compression levels
- [x] **Cross-Platform**: Supports Linux, macOS, and Windows with platform-specific notifications
- [x] **Low Overhead**: Minimal system resource usage with efficient process scanning
- [x] **Configurable**: Flexible settings for thresholds and check intervals
- [x] **Smart Notification System**: Built-in cooldown period to prevent notification spam

## Installation

### Prerequisites

- Rust 1.85 or higher
- Cargo package manager
- Platform-specific dependencies:
  - Linux: `libdbus-1-dev` (for notifications via notify-rust)
  - macOS: None (uses osascript for notifications)
  - Windows: None (uses notify-rust for notifications)

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

### From Release

see [https://github.com/xosnrdev/data-guardian/releases](https://github.com/xosnrdev/data-guardian/releases)

## Usage

### Quick Start

1. Start the service:

   ```bash
   dg
   ```

2. The service will run in the background and monitor disk I/O usage of all processes

3. Receive notifications when applications exceed configured thresholds

### Configuration

The service can be configured in three ways (in order of precedence):

1. Environment variables:

   ```bash
   DATAGUARDIAN_DATA_LIMIT=1073741824
   DATAGUARDIAN_CHECK_INTERVAL_SECONDS=60
   DATAGUARDIAN_PERSISTENCE_INTERVAL_SECONDS=300
   ```

2. User configuration file at:
   - Linux: `~/.config/DataGuardian/config.toml`
   - macOS: `~/Library/Application Support/DataGuardian/config.toml`
   - Windows: `%APPDATA%\DataGuardian\config.toml`

   Example `config.toml`:

   ```toml
   # Data limit in bytes before triggering alerts
   data_limit = 1073741824  # 1 GB

   # How often to check process data usage (in seconds)
   check_interval_seconds = 60

   # How often to save usage data to disk (in seconds)
   persistence_interval_seconds = 300  # 5 minutes
   ```

3. Default values:
   - `data_limit`: 1 GB (1073741824 bytes)
   - `check_interval_seconds`: 60 seconds
   - `persistence_interval_seconds`: 300 seconds (5 minutes)

### Environment Variables

- `DATAGUARDIAN_DATA_LIMIT`: Override data limit (minimum: 1MB)
- `DATAGUARDIAN_CHECK_INTERVAL_SECONDS`: Override check interval (minimum: 1 second)
- `DATAGUARDIAN_PERSISTENCE_INTERVAL_SECONDS`: Override persistence interval (minimum: 10 seconds)
- `RUST_LOG`: Set logging level (error, warn, info, debug, trace)

### Limitations

- Only tracks processes while the service is running
- Uses a single global threshold rather than per-application limits
- May have different behavior on different platforms
- Focuses only on disk I/O (not network or other resource usage)

## License

[MIT License](LICENSE)
