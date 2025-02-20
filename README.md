# Data Guardian

A system service for monitoring and optimizing application network usage.

## Features

- Real-time per-application network monitoring
- Cross-platform data limit notifications
- Usage persistence between restarts
- Configurable thresholds and check intervals

## Installation

```bash
cargo install --path .
```

## Configuration

1. Copy `config/default.toml` to `~/.config/DataGuardian/local.toml`
2. Modify values as needed
3. Environment variables can override config using `DATAGUARDIAN_` prefix

## Usage

```bash
dg # Requires root/sudo privileges for network monitoring
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT

