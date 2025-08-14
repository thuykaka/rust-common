//! # Logger Error Module
//!
//! Provides custom error types for the logger system using `thiserror` and `anyhow`.
//!
//! ## Examples
//!
//! ```rust
//! use rust_common::logger::LoggerError;
//! use anyhow::Context;
//!
//! // Create different types of logger errors
//! let config_error = LoggerError::InvalidConfiguration("Missing log directory".to_string());
//! let init_error = LoggerError::InitializationFailed("Tracing subscriber already set".to_string());
//!
//! // Using anyhow for error context
//! # fn some_operation() -> anyhow::Result<()> { Ok(()) }
//! let result = some_operation().context("Failed to initialize logger")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use thiserror::Error;

/// Custom error types for the logger system
#[derive(Error, Debug)]
pub enum LoggerError {
    /// Configuration validation error
    #[error("Invalid logger configuration: {0}")]
    InvalidConfiguration(String),

    /// Logger initialization error
    #[error("Logger initialization failed: {0}")]
    InitializationFailed(String),

    /// I/O related error
    #[error("Logger I/O error")]
    IoError(#[from] std::io::Error),

    /// Tracing subscriber error
    #[error("Tracing subscriber error")]
    TracingError(#[from] tracing_subscriber::util::TryInitError),

    /// Generic error with context
    #[error("Logger error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias for logger operations using anyhow
pub type LoggerResult<T> = anyhow::Result<T>;

/// Validates logger configuration
pub fn validate_config(config: &crate::logger::LoggerConfig) -> LoggerResult<()> {
    use anyhow::bail;

    // Check if at least one output is enabled
    if !config.enable_console() && !config.enable_file() {
        bail!(LoggerError::InvalidConfiguration(
            "Must enable at least one of file or console logging".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::LoggerConfig;

    #[test]
    fn test_error_display() {
        let config_error = LoggerError::InvalidConfiguration("test error".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Invalid logger configuration: test error"
        );

        let init_error = LoggerError::InitializationFailed("init failed".to_string());
        assert_eq!(
            format!("{}", init_error),
            "Logger initialization failed: init failed"
        );

        let io_error = LoggerError::IoError(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
        assert!(format!("{}", io_error).contains("Logger I/O error"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let logger_err = LoggerError::from(io_err);

        match logger_err {
            LoggerError::IoError(_) => (),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_validate_config_success() {
        let config = LoggerConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_no_outputs() {
        let config = LoggerConfig::builder()
            .enable_console(false)
            .enable_file(false)
            .build();

        let result = validate_config(&config);
        assert!(result.is_err());

        // Check if the error contains the expected message
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Must enable at least one"));
    }


    #[test]
    fn test_validate_config_console_only() {
        let config = LoggerConfig::builder()
            .enable_console(true)
            .enable_file(false)
            .build();

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_file_only() {
        let config = LoggerConfig::builder()
            .enable_console(false)
            .enable_file(true)
            .log_dir("logs")
            .log_filename("app.log")
            .build();

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_anyhow_integration() {
        use anyhow::Context;

        // Test that we can use anyhow context with our errors
        let config = LoggerConfig::builder()
            .enable_console(false)
            .enable_file(false)
            .build();

        let result = validate_config(&config).context("Failed to validate logger configuration");

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Failed to validate logger configuration"));
    }

    #[test]
    fn test_thiserror_from_traits() {
        // Test automatic conversion from std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let logger_err: LoggerError = io_err.into();

        match logger_err {
            LoggerError::IoError(_) => (),
            _ => panic!("Expected IoError variant"),
        }
    }
}
