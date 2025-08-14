//! # Logger Module
//!
//! Provides structured logging capabilities with flexible configuration options.
//!
//! ## Features
//!
//! - Structured logging with tracing
//! - File and console output support
//! - Configurable log levels and formatting
//! - Rolling file appenders
//! - Async-friendly logging
//! - Custom error types
//!
//! ## Examples
//!
//! ```rust,no_run
//! use rust_common::logger;
//! use tracing::{info, error};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Initialize with default configuration
//! logger::init_with_default()?;
//!
//! // Log structured data
//! info!(
//!     user_id = 12345,
//!     action = "login",
//!     ip = "192.168.1.1",
//!     "User performed action"
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ```rust,no_run
//! use rust_common::logger;
//! use tracing::info;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Custom configuration
//! let config = logger::LoggerConfig::builder()
//!     .default_level(tracing::Level::DEBUG)
//!     .log_dir("custom_logs")
//!     .enable_console(true)
//!     .enable_file(false)
//!     .build();
//!
//! logger::init(config)?;
//! info!("Logger configured successfully");
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod init;

// Re-export main types and functions
pub use config::*;
pub use error::*;
pub use init::*;

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn test_default_config() {
        let config = LoggerConfig::default();
        assert_eq!(config.default_level(), Level::INFO);
        assert_eq!(config.log_dir(), "logs");
        assert_eq!(config.log_filename(), "application.log");
        assert!(config.enable_console());
        assert!(config.enable_file());
    }

    #[test]
    fn test_config_builder() {
        let config = LoggerConfig::builder()
            .default_level(Level::DEBUG)
            .log_dir("test_logs")
            .log_filename("test.log")
            .enable_console(false)
            .enable_file(true)
            .build();

        assert_eq!(config.default_level(), Level::DEBUG);
        assert_eq!(config.log_dir(), "test_logs");
        assert_eq!(config.log_filename(), "test.log");
        assert!(!config.enable_console());
        assert!(config.enable_file());
    }

    #[test]
    fn test_module_structure() {
        // Test that all sub-modules are accessible
        let _config = config::LoggerConfig::default();
        let _error = error::LoggerError::InvalidConfiguration("test".to_string());
    }
}
