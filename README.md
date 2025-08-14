# Rust Common Library

A comprehensive utility library for Rust projects providing common functionality across different domains including logging and more.

## Features

- **Logger**: Structured logging with tracing
- **Extensible**: Easy to add new modules
- **Well-tested**: Comprehensive test coverage
- **Documented**: Full documentation with examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-common = "0.1.0"
```

## Installation

```bash
# Install with cargo
cargo add rust-common
```

## Usage

### Basic Usage

```rust
use rust_common::logger;
use tracing::info;

// Initialize logger
logger::init_with_default()?;

info!("Logger initialized successfully");
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Using the Prelude

For convenience, you can use the prelude module:

```rust
use rust_common::prelude::*;

// Initialize logger with default configuration
init_with_default()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Modules

### Logger Module

The logger module provides structured logging with tracing:

#### Configuration

- `LoggerConfig` - Configuration for the logger
- `init_with_default()` - Initialize with default configuration
- `init_with_config(config)` - Initialize with custom configuration

#### Features

- File and console logging
- Configurable log levels
- Structured logging with tracing
- Error handling integration

## Examples

### Logger Usage

```rust
use rust_common::logger;
use tracing::{info, warn, error};

// Initialize logger
logger::init_with_default()?;

// Use structured logging
info!("Application started");
warn!("This is a warning");
error!("An error occurred");

# Ok::<(), Box<dyn std::error::Error>>(())
```

### Custom Configuration

```rust
use rust_common::logger::LoggerConfig;

let config = LoggerConfig::new()
    .with_log_dir("custom_logs")
    .with_log_filename("app.log")
    .with_console_enabled(true);

logger::init_with_config(config)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Development

### Running Tests

```bash
cargo test
```

### Running Documentation

```bash
cargo doc --open
```

### Building

```bash
cargo build
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [ ] Add data structure utilities
- [ ] Add string manipulation utilities
- [ ] Add date/time utilities
- [ ] Add file I/O utilities
- [ ] Add networking utilities
- [ ] Add configuration management utilities
