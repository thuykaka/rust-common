//! # Logger Configuration Module
//!
//! Provides configuration structures and builders for the logger system.
//!
//! ## Examples
//!
//! ```rust
//! use rust_common::logger::LoggerConfig;
//! use tracing::Level;
//! use tracing_appender::rolling::Rotation;
//!
//! // Using default configuration
//! let config = LoggerConfig::default();
//!
//! // Using builder pattern
//! let config = LoggerConfig::builder()
//!     .default_level(Level::DEBUG)
//!     .log_dir("custom_logs")
//!     .log_filename("app.log")
//!     .show_file_line(true)
//!     .show_thread(false)
//!     .show_target(true)
//!     .use_ansi(true)
//!     .enable_console(true)
//!     .enable_file(true)
//!     .rotation(Rotation::HOURLY)
//!     .show_spans(false)
//!     .build();
//! ```

use tracing::Level;
use tracing_appender::rolling::Rotation;

/// Configuration for the logger system
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    default_level: Level,
    log_dir: String,
    log_filename: String,
    show_file_line: bool,
    show_thread: bool,
    show_target: bool,
    use_ansi: bool,
    enable_console: bool,
    enable_file: bool,
    rotation: Rotation,
    show_spans: bool,
}

impl LoggerConfig {
    /// Creates a new builder for LoggerConfig
    pub fn builder() -> LoggerConfigBuilder {
        LoggerConfigBuilder::new()
    }

    /// Gets the default log level
    pub fn default_level(&self) -> Level {
        self.default_level
    }

    /// Gets the log directory
    pub fn log_dir(&self) -> &str {
        &self.log_dir
    }

    /// Gets the log filename
    pub fn log_filename(&self) -> &str {
        &self.log_filename
    }

    /// Gets whether to show file and line numbers
    pub fn show_file_line(&self) -> bool {
        self.show_file_line
    }

    /// Gets whether to show thread information
    pub fn show_thread(&self) -> bool {
        self.show_thread
    }

    /// Gets whether to show target information
    pub fn show_target(&self) -> bool {
        self.show_target
    }

    /// Gets whether to use ANSI colors
    pub fn use_ansi(&self) -> bool {
        self.use_ansi
    }

    /// Gets whether console logging is enabled
    pub fn enable_console(&self) -> bool {
        self.enable_console
    }

    /// Gets whether file logging is enabled
    pub fn enable_file(&self) -> bool {
        self.enable_file
    }

    /// Gets the file rotation strategy
    pub fn rotation(&self) -> Rotation {
        self.rotation.clone()
    }

    /// Gets whether to show span events
    pub fn show_spans(&self) -> bool {
        self.show_spans
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            default_level: Level::INFO,
            log_dir: "logs".to_string(),
            log_filename: "application.log".to_string(),
            show_file_line: cfg!(debug_assertions),
            show_thread: false,
            show_target: false,
            use_ansi: true,
            enable_console: true,
            enable_file: true,
            rotation: Rotation::DAILY,
            show_spans: false,
        }
    }
}

/// Builder for LoggerConfig
#[derive(Debug)]
pub struct LoggerConfigBuilder {
    config: LoggerConfig,
}

impl LoggerConfigBuilder {
    /// Creates a new builder with default values
    pub fn new() -> Self {
        Self {
            config: LoggerConfig::default(),
        }
    }

    /// Sets the default log level
    pub fn default_level(mut self, level: Level) -> Self {
        self.config.default_level = level;
        self
    }

    /// Sets the log directory
    pub fn log_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.config.log_dir = dir.into();
        self
    }

    /// Sets the log filename
    pub fn log_filename<S: Into<String>>(mut self, filename: S) -> Self {
        self.config.log_filename = filename.into();
        self
    }

    /// Sets whether to show file and line numbers
    pub fn show_file_line(mut self, show: bool) -> Self {
        self.config.show_file_line = show;
        self
    }

    /// Sets whether to show thread information
    pub fn show_thread(mut self, show: bool) -> Self {
        self.config.show_thread = show;
        self
    }

    /// Sets whether to show target information
    pub fn show_target(mut self, show: bool) -> Self {
        self.config.show_target = show;
        self
    }

    /// Sets whether to use ANSI colors
    pub fn use_ansi(mut self, use_ansi: bool) -> Self {
        self.config.use_ansi = use_ansi;
        self
    }

    /// Sets whether console logging is enabled
    pub fn enable_console(mut self, enable: bool) -> Self {
        self.config.enable_console = enable;
        self
    }

    /// Sets whether file logging is enabled
    pub fn enable_file(mut self, enable: bool) -> Self {
        self.config.enable_file = enable;
        self
    }

    /// Sets the file rotation strategy
    pub fn rotation(mut self, rotation: Rotation) -> Self {
        self.config.rotation = rotation;
        self
    }

    /// Sets whether to show span events
    pub fn show_spans(mut self, show: bool) -> Self {
        self.config.show_spans = show;
        self
    }

    /// Builds the LoggerConfig
    pub fn build(self) -> LoggerConfig {
        self.config
    }
}

impl Default for LoggerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggerConfig::default();
        assert_eq!(config.default_level(), Level::INFO);
        assert_eq!(config.log_dir(), "logs");
        assert_eq!(config.log_filename(), "application.log");
        assert!(config.enable_console());
        assert!(config.enable_file());
        assert!(!config.show_spans());
    }

    #[test]
    fn test_builder_pattern() {
        let config = LoggerConfig::builder()
            .default_level(Level::DEBUG)
            .log_dir("test_logs")
            .log_filename("test.log")
            .show_file_line(true)
            .show_thread(true)
            .show_target(true)
            .use_ansi(false)
            .enable_console(false)
            .enable_file(true)
            .rotation(Rotation::HOURLY)
            .show_spans(true)
            .build();

        assert_eq!(config.default_level(), Level::DEBUG);
        assert_eq!(config.log_dir(), "test_logs");
        assert_eq!(config.log_filename(), "test.log");
        assert!(config.show_file_line());
        assert!(config.show_thread());
        assert!(config.show_target());
        assert!(!config.use_ansi());
        assert!(!config.enable_console());
        assert!(config.enable_file());
        assert!(config.show_spans());
    }

    #[test]
    fn test_builder_default() {
        let builder = LoggerConfigBuilder::default();
        let config = builder.build();

        // Should match LoggerConfig::default()
        let default_config = LoggerConfig::default();
        assert_eq!(config.default_level(), default_config.default_level());
        assert_eq!(config.log_dir(), default_config.log_dir());
        assert_eq!(config.log_filename(), default_config.log_filename());
    }

    #[test]
    fn test_config_getters() {
        let config = LoggerConfig::builder()
            .default_level(Level::WARN)
            .log_dir("custom")
            .log_filename("custom.log")
            .build();

        assert_eq!(config.default_level(), Level::WARN);
        assert_eq!(config.log_dir(), "custom");
        assert_eq!(config.log_filename(), "custom.log");
    }
}
