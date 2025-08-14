//! # Logger Demo
//!
//! Demonstrates basic usage of the rust-common logger module

use anyhow::Context;
use rust_common::logger::{self};
use tracing::{error, info, instrument};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

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

    info!("Data: {}", data);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum DemoError {
    #[error("Cannot process id {0}")]
    CannotProcessId(i32),
}

#[instrument]
async fn process_data(id: i32) -> Result<i32, DemoError> {
    info!("Bắt đầu xử lý dữ liệu");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    if id == 42 {
        return Err(DemoError::CannotProcessId(id));
    }

    info!("Dữ liệu đã được xử lý thành công");
    Ok(id)
}
