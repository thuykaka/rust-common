//! # Rust Common Library
//!
//! A comprehensive utility library for Rust projects providing common functionality
//! across different domains including logging and more.
//!
//! ## Features
//!
//! - **Logger**: Structured logging with tracing
//! - **Extensible**: Easy to add new modules
//! - **Well-tested**: Comprehensive test coverage
//!
//! ## Usage
//!
//! ```rust
//! use rust_common::logger;
//! use tracing::info;
//!
//! // Initialize logger
//! logger::init_with_default()?;
//!
//! info!("Logger initialized successfully");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Modules
//!
//! - `logger`: Structured logging with tracing

// Logger module is always available
pub mod kafka;
pub mod logger;

/// Re-export commonly used items for convenience
pub mod prelude {
    // Re-export specific types to avoid naming conflicts
    pub use crate::kafka::{HandlerResult, KafkaClientConfig, KafkaError, StreamHandler};
    pub use crate::logger::{init_with_default, LoggerConfig};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_module() {
        // Test that logger module is accessible
        let config = logger::LoggerConfig::default();
        assert_eq!(config.log_dir(), "logs");
        assert_eq!(config.log_filename(), "application.log");
    }

    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;

        // Test logger types from prelude
        let config = LoggerConfig::default();
        assert!(config.enable_console());
    }
}
