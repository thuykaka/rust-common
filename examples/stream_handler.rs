use anyhow::Context;
use rust_common::kafka::{
    config::KafkaConfig,
    stream_handler::{HandlerResult, StreamHandler},
};
use rust_common::logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    let kafka_config = KafkaConfig {
        client_id: None,
        cluster_id: "aaa".to_string(),
        bootstrap_servers: "localhost:9092".to_string(),
        topics: vec!["aaa".to_string()],
    };

    let mut stream_handler = StreamHandler::new(kafka_config)?;

    stream_handler.register("/api/v1/login", |msg| async move {

        // Use async sleep instead of blocking sleep
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok(HandlerResult::Response(serde_json::json!({
            "token": "123",
            "sent": msg.data.clone()
        })))
    })?;

    stream_handler.start_processing().await?;

    Ok(())
}
