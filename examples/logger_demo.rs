use anyhow::Context;
use std::io;
use tracing::{error, info, instrument, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, registry, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone)]
struct LoggerConfig {
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

fn init_logger_with_default() -> anyhow::Result<()> {
    let config = LoggerConfig::default();
    init_logger(config)
}

fn init_logger(config: LoggerConfig) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.default_level.as_str()));

    let time_format = "%Y-%m-%d %H:%M:%S%.3f";

    match (config.enable_file, config.enable_console) {
        (true, true) => {
            let file_appender =
                RollingFileAppender::new(config.rotation, &config.log_dir, &config.log_filename);

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_file(config.show_file_line)
                .with_line_number(config.show_file_line)
                .with_thread_ids(config.show_thread)
                .with_thread_names(config.show_thread)
                .with_target(config.show_target)
                .with_span_events(if config.show_spans {
                    fmt::format::FmtSpan::FULL
                } else {
                    fmt::format::FmtSpan::NONE
                })
                .with_timer(fmt::time::ChronoLocal::new(time_format.to_string()));

            let console_layer = fmt::layer()
                .with_writer(io::stdout)
                .with_ansi(config.use_ansi)
                .with_file(config.show_file_line)
                .with_line_number(config.show_file_line)
                .with_thread_ids(config.show_thread)
                .with_thread_names(config.show_thread)
                .with_target(config.show_target)
                .with_span_events(if config.show_spans {
                    fmt::format::FmtSpan::FULL
                } else {
                    fmt::format::FmtSpan::NONE
                })
                .with_timer(fmt::time::ChronoLocal::new(time_format.to_string()));

            registry()
                .with(env_filter)
                .with(file_layer)
                .with(console_layer)
                .try_init()?;
        }
        (true, false) => {
            let file_appender =
                RollingFileAppender::new(config.rotation, &config.log_dir, &config.log_filename);

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_file(config.show_file_line)
                .with_line_number(config.show_file_line)
                .with_thread_ids(config.show_thread)
                .with_thread_names(config.show_thread)
                .with_target(config.show_target)
                .with_span_events(if config.show_spans {
                    fmt::format::FmtSpan::FULL
                } else {
                    fmt::format::FmtSpan::NONE
                })
                .with_timer(fmt::time::ChronoLocal::new(time_format.to_string()));

            registry().with(env_filter).with(file_layer).try_init()?;
        }
        (false, true) => {
            let console_layer = fmt::layer()
                .with_writer(io::stdout)
                .with_ansi(config.use_ansi)
                .with_file(config.show_file_line)
                .with_line_number(config.show_file_line)
                .with_thread_ids(config.show_thread)
                .with_thread_names(config.show_thread)
                .with_target(config.show_target)
                .with_span_events(if config.show_spans {
                    fmt::format::FmtSpan::FULL
                } else {
                    fmt::format::FmtSpan::NONE
                })
                .with_timer(fmt::time::ChronoLocal::new(time_format.to_string()));

            registry().with(env_filter).with(console_layer).try_init()?;
        }
        (false, false) => {
            return Err(anyhow::anyhow!(
                "Must enable at least one of file or console logging"
            ))
            .context("Cannot initialize logger");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Khởi tạo logger
    init_logger_with_default()?;

    // Demo structured logging
    let user_id = 12345;
    let action = "login";
    info!(
        user_id = user_id,
        action = action,
        ip = "192.168.1.1",
        "User thực hiện hành động"
    );

    let data = process_data(42).await.unwrap_or_else(|e| {
        error!(error = e.to_string(), "Lỗi khi xử lý dữ liệu");
        -1
    });

    info!("Kết quả cuối cùng: {}", data);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum DemoThisError {
    #[error("Cannot process id {0}")]
    CannotProcessId(i32),
}

#[instrument]
async fn process_data(id: i32) -> Result<i32, DemoThisError> {
    info!("Bắt đầu xử lý dữ liệu");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    if id == 42 {
        return Err(DemoThisError::CannotProcessId(id));
    }

    info!("Dữ liệu đã được xử lý thành công");
    Ok(id)
}
