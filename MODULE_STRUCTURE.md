# Module Structure Documentation

## Logger Module Organization

The `logger` module is organized for easy management and extensibility:

```
src/logger/
├── mod.rs           # Entry point, re-exports all sub-modules
├── config.rs        # Logger configuration
├── init.rs          # Logger initialization
└── error.rs         # Logger error types
```

## Usage Methods

### 1. Direct usage from main module

```rust
use rust_common::logger;

logger::init_with_default()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 2. Using specific sub-modules

```rust
use rust_common::logger::config::LoggerConfig;
use rust_common::logger::init;

let config = LoggerConfig::default();
init::init_with_config(config)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 3. Import all from the logger module

```rust
use rust_common::logger::*;

init_with_default()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Sub-modules

### `config.rs`

- `LoggerConfig` - Configuration struct for logger
- `with_log_dir()` - Set log directory
- `with_log_filename()` - Set log filename
- `with_console_enabled()` - Enable/disable console logging

### `init.rs`

- `init_with_default()` - Initialize with default configuration
- `init_with_config(config)` - Initialize with custom configuration

### `error.rs`

- `LoggerError` - Error types for logger operations
- Error handling for initialization failures

## Benefits of this structure

1. **Clear organization**: Each file contains related functionality
2. **Easy maintenance**: Modifying one component doesn't affect others
3. **Easy extensibility**: Adding new logger features is straightforward
4. **Flexibility**: Can import individual modules or all at once
5. **Error handling**: Dedicated error types for better debugging

## Adding a new module

To add a new module (e.g., `formatter.rs`):

1. Create file `src/logger/formatter.rs`
2. Add `pub mod formatter;` to `src/logger/mod.rs`
3. Add `pub use formatter::*;` to `src/logger/mod.rs`
4. Write formatter functions in the new file
