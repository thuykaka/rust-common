# Features Documentation

## Overview of Features

The `rust-common` library focuses on providing essential utilities for Rust projects, starting with structured logging capabilities.

## Current Features

### Logger Module

**Module included:**

- `logger` - Structured logging with tracing

**Available functionality:**

```rust
// Configuration
LoggerConfig::default()
LoggerConfig::new()
    .with_log_dir("logs")
    .with_log_filename("app.log")
    .with_console_enabled(true)

// Initialization
init_with_default()
init_with_config(config)
```

## Usage

### Installation

```toml
[dependencies]
rust-common = "0.1.0"
```

### Installation via command line

```bash
# Install the library
cargo add rust-common
```

## Logger Features

### Configuration Options

The logger supports various configuration options:

```rust
use rust_common::logger::LoggerConfig;

let config = LoggerConfig::new()
    .with_log_dir("custom_logs")        // Set custom log directory
    .with_log_filename("my_app.log")    // Set custom log filename
    .with_console_enabled(true);        // Enable console output
```

### Initialization

```rust
use rust_common::logger;

// Initialize with default settings
logger::init_with_default()?;

// Or initialize with custom configuration
let config = LoggerConfig::new()
    .with_log_dir("logs")
    .with_console_enabled(true);
logger::init_with_config(config)?;
```

### Usage with Tracing

```rust
use tracing::{info, warn, error, debug, trace};

// After initialization, use tracing macros
info!("Application started");
warn!("This is a warning message");
error!("An error occurred: {}", error_msg);
debug!("Debug information");
trace!("Detailed trace information");
```

## Benefits of the Current Structure

1. **Focused functionality**: Concentrates on essential logging capabilities
2. **Easy to use**: Simple initialization and configuration
3. **Flexible**: Supports both file and console logging
4. **Structured**: Uses tracing for structured logging
5. **Error handling**: Proper error types with thiserror integration

## Future Roadmap

The library is designed to be extensible. Future modules may include:

- **Configuration management**: Environment and file-based configuration
- **String utilities**: Common string manipulation functions
- **Date/time utilities**: Date and time handling helpers
- **File I/O utilities**: File system operation helpers
- **Data structures**: Common data structure implementations
- **Networking utilities**: HTTP client/server helpers

## Adding New Features

To add a new feature module:

1. Create the module directory under `src/`
2. Implement the module functionality
3. Add the module to `src/lib.rs`
4. Update documentation
5. Add examples and tests

The current architecture supports easy extension while maintaining backward compatibility.
