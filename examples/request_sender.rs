use anyhow::Context;
use rust_common::{
    kafka::{
        request_sender::{RequestAsyncParams, RequestSender},
        KafkaClientConfig,
    },
    logger,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    let cluster_id = "xxx-yyy".to_string();
    let bootstrap_servers = "localhost:9092".to_string();

    let config = KafkaClientConfig::new(cluster_id, bootstrap_servers);

    tracing::info!("init request sender with config: {:?}", config);

    let request_sender = RequestSender::new(config)?;

    request_sender.start().await?;

    let params = RequestAsyncParams::new(
        "test-service".to_string(),
        "/api/v1/demo".to_string(),
        Some("message-id".to_string()),
        serde_json::json!({
            "name": "John Doe",
            "age": 30,
        }),
    )
    .with_timeout_secs(10);

    let response = request_sender.send_request_async(params).await?;

    tracing::info!("response: {:?}", response);

    Ok(())
}
