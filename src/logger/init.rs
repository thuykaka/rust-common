//! # Logger Initialization Module
//!
//! Provides functions to initialize the logger system with various configurations.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use rust_common::logger;
//! use tracing::{info, Level};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Initialize with default configuration
//! logger::init_with_default()?;
//! info!("Logger initialized with defaults");
//! # Ok(())
//! # }
//! ```
//!
//! ```rust,no_run
//! use rust_common::logger;
//! use tracing::{info, Level};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Initialize with custom configuration
//! let config = logger::LoggerConfig::builder()
//!     .default_level(Level::DEBUG)
//!     .log_dir("custom_logs")
//!     .enable_console(true)
//!     .enable_file(false)
//!     .build();
//!
//! logger::init(config)?;
//! info!("Logger initialized with custom config");
//! # Ok(())
//! # }
//! ```

use crate::logger::{error::validate_config, LoggerConfig, LoggerError, LoggerResult};
use anyhow::Context;
use std::io;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{fmt, prelude::*, registry, util::SubscriberInitExt, EnvFilter};

/// Initializes the logger with default configuration
///
/// This is a convenience function that creates a default `LoggerConfig`
/// and initializes the logger system.
///
/// # Returns
///
/// `LoggerResult<()>` - Ok(()) if initialization succeeds, Err otherwise
///
/// # Examples
///
/// ```rust
/// use rust_common::logger;
/// use tracing::info;
///
/// // Initialize logger with defaults
/// logger::init_with_default()?;
/// info!("Logger is ready!");
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn init_with_default() -> LoggerResult<()> {
    let config = LoggerConfig::default();
    init(config)
}

/// Initializes the logger with the provided configuration
///
/// This function sets up the tracing subscriber with the specified configuration,
/// including file and/or console output, formatting options, and log levels.
///
/// # Arguments
///
/// * `config` - The logger configuration to use
///
/// # Returns
///
/// `LoggerResult<()>` - Ok(()) if initialization succeeds, Err otherwise
///
/// # Examples
///
/// ```rust
/// use rust_common::logger::{LoggerConfig, init};
/// use tracing::{Level, info};
/// use tracing_appender::rolling::Rotation;
///
/// let config = LoggerConfig::builder()
///     .default_level(Level::DEBUG)
///     .log_dir("app_logs")
///     .log_filename("debug.log")
///     .show_file_line(true)
///     .rotation(Rotation::HOURLY)
///     .build();
///
/// init(config)?;
/// info!("Logger initialized successfully");
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn init(config: LoggerConfig) -> LoggerResult<()> {
    // Validate configuration first
    validate_config(&config)?;

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.default_level().as_str()));

    // Time format for logs
    let time_format = "%Y-%m-%d %H:%M:%S%.3f";

    // Initialize based on enabled outputs
    match (config.enable_file(), config.enable_console()) {
        (true, true) => init_with_both_outputs(&config, env_filter, time_format)
            .context("Failed to initialize logger with both file and console output"),
        (true, false) => init_with_file_output(&config, env_filter, time_format)
            .context("Failed to initialize logger with file output"),
        (false, true) => init_with_console_output(&config, env_filter, time_format)
            .context("Failed to initialize logger with console output"),
        (false, false) => Err(LoggerError::InvalidConfiguration(
            "Must enable at least one of file or console logging".to_string(),
        )
        .into()),
    }
}

/// Initializes logger with both file and console output
fn init_with_both_outputs(
    config: &LoggerConfig,
    env_filter: EnvFilter,
    time_format: &str,
) -> LoggerResult<()> {
    let file_appender =
        RollingFileAppender::new(config.rotation(), config.log_dir(), config.log_filename());

    let file_layer = create_file_layer(file_appender, config, time_format);
    let console_layer = create_console_layer(config, time_format);

    registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| LoggerError::TracingError(e).into())
}

/// Initializes logger with file output only
fn init_with_file_output(
    config: &LoggerConfig,
    env_filter: EnvFilter,
    time_format: &str,
) -> LoggerResult<()> {
    let file_appender =
        RollingFileAppender::new(config.rotation(), config.log_dir(), config.log_filename());

    let file_layer = create_file_layer(file_appender, config, time_format);

    registry()
        .with(env_filter)
        .with(file_layer)
        .try_init()
        .map_err(|e| LoggerError::TracingError(e).into())
}

/// Initializes logger with console output only
fn init_with_console_output(
    config: &LoggerConfig,
    env_filter: EnvFilter,
    time_format: &str,
) -> LoggerResult<()> {
    let console_layer = create_console_layer(config, time_format);

    registry()
        .with(env_filter)
        .with(console_layer)
        .try_init()
        .map_err(|e| LoggerError::TracingError(e).into())
}

/// Creates a file logging layer
fn create_file_layer<S>(
    file_appender: RollingFileAppender,
    config: &LoggerConfig,
    time_format: &str,
) -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    Box::new(
        fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false) // No ANSI colors in files
            .with_file(config.show_file_line())
            .with_line_number(config.show_file_line())
            .with_thread_ids(config.show_thread())
            .with_thread_names(config.show_thread())
            .with_target(config.show_target())
            .with_span_events(if config.show_spans() {
                fmt::format::FmtSpan::FULL
            } else {
                fmt::format::FmtSpan::NONE
            })
            .with_timer(fmt::time::ChronoLocal::new(time_format.to_string())),
    )
}

/// Creates a console logging layer
fn create_console_layer<S>(
    config: &LoggerConfig,
    time_format: &str,
) -> Box<dyn tracing_subscriber::Layer<S> + Send + Sync + 'static>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    Box::new(
        fmt::layer()
            .with_writer(io::stdout)
            .with_ansi(config.use_ansi())
            .with_file(config.show_file_line())
            .with_line_number(config.show_file_line())
            .with_thread_ids(config.show_thread())
            .with_thread_names(config.show_thread())
            .with_target(config.show_target())
            .with_span_events(if config.show_spans() {
                fmt::format::FmtSpan::FULL
            } else {
                fmt::format::FmtSpan::NONE
            })
            .with_timer(fmt::time::ChronoLocal::new(time_format.to_string())),
    )
}

/// Checks if a logger has already been initialized
///
/// This function attempts to initialize a no-op subscriber to check
/// if a global subscriber has already been set.
///
/// # Returns
///
/// `true` if a logger is already initialized, `false` otherwise
pub fn is_initialized() -> bool {
    tracing_subscriber::registry().try_init().is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;
    use tracing_appender::rolling::Rotation;

    #[test]
    fn test_init_with_default() {
        // Note: This test may fail if run after other tests that initialize the logger
        // In a real application, you would only initialize once
        let result = init_with_default();
        // We can't assert success because the logger might already be initialized
        // from other tests, so we just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_init_with_custom_config() {
        let config = LoggerConfig::builder()
            .default_level(Level::DEBUG)
            .log_dir("test_logs")
            .log_filename("test.log")
            .enable_console(true)
            .enable_file(false) // Disable file to avoid creating actual files in tests
            .build();

        let result = init(config);
        // We can't assert success because the logger might already be initialized
        let _ = result;
    }

    #[test]
    fn test_init_with_invalid_config() {
        let config = LoggerConfig::builder()
            .enable_console(false)
            .enable_file(false)
            .build();

        let result = init(config);
        assert!(result.is_err());

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Must enable at least one"));
    }

    #[test]
    fn test_create_file_layer() {
        let config = LoggerConfig::builder()
            .show_file_line(true)
            .show_thread(true)
            .show_target(true)
            .show_spans(true)
            .build();

        let file_appender = RollingFileAppender::new(Rotation::DAILY, "test_logs", "test.log");

        let _layer: Box<dyn tracing_subscriber::Layer<registry::Registry> + Send + Sync> =
            create_file_layer(file_appender, &config, "%Y-%m-%d %H:%M:%S");
        // If we get here without panicking, the layer was created successfully
    }

    #[test]
    fn test_create_console_layer() {
        let config = LoggerConfig::builder()
            .use_ansi(true)
            .show_file_line(false)
            .show_thread(false)
            .show_target(false)
            .show_spans(false)
            .build();

        let _layer: Box<dyn tracing_subscriber::Layer<registry::Registry> + Send + Sync> =
            create_console_layer(&config, "%Y-%m-%d %H:%M:%S");
        // If we get here without panicking, the layer was created successfully
    }

    #[test]
    fn test_is_initialized() {
        // This function should work regardless of initialization state
        let _initialized = is_initialized();
        // We can't assert a specific value because it depends on test execution order
    }
}
