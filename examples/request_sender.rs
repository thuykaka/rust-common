use anyhow::Context;
use rust_common::{
    kafka::{
        request_sender::{RequestAsyncParams, RequestSender},
        KafkaClientConfig,
    },
    logger,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::signal;

#[derive(Serialize, Deserialize, Debug)]
pub struct DemoData {
    pub demo: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    let cluster_id = "xxx-yyy".to_string();
    let bootstrap_servers = "localhost:9092".to_string();

    let config = KafkaClientConfig::new(cluster_id, bootstrap_servers);

    tracing::info!("init request sender with config: {:?}", config);

    let request_sender = Arc::new(RequestSender::new(config)?);

    let background_task = request_sender.start().await?;

    // Gửi 1000 messages song song như Promise.all
    let start_time = std::time::Instant::now();

    let futures = (0..10_000)
        .map(|i| {
            let request_sender = Arc::clone(&request_sender);
            let params = RequestAsyncParams::new(
                "test-service-rust".to_string(),
                "/api/v1/login".to_string(),
                Some(format!("message-id-{}", i)),
                serde_json::json!({
                    "name": format!("User {}", i),
                    "age": 20 + (i % 50),
                }),
            )
            .with_timeout_secs(300);

            tokio::spawn(async move {
                let result = request_sender.send_request_async(params).await;
                (i, result)
            })
        })
        .collect::<Vec<_>>();

    tracing::info!("Sending 10000 requests concurrently...");

    // Đợi tất cả futures hoàn thành (tương đương Promise.all)
    let results = futures::future::join_all(futures).await;

    let mut success_count = 0;
    let mut error_count = 0;

    for join_result in results {
        match join_result {
            Ok((i, request_result)) => {
                match request_result {
                    Ok(response) => {
                        success_count += 1;
                        if i < 5 {
                            // Log first 5 responses
                            tracing::info!(
                                "Request {}: {:?}",
                                i,
                                response.get_data_as::<DemoData>()
                            );
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        tracing::error!("Request {} failed: {:?}", i, e);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                tracing::error!("Task join error: {:?}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    tracing::info!(
        "Completed 1000 requests in {:?} - Success: {}, Errors: {}",
        duration,
        success_count,
        error_count
    );

    signal::ctrl_c().await?;
    tracing::info!("Shutting down...");
    background_task.abort();

    Ok(())
}
