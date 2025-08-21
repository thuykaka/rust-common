use anyhow::Context;
use rust_common::kafka::core::KafkaClientConfig;
use rust_common::logger;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    let cluster_id = "test-service-rust".to_string();
    let bootstrap_servers = "localhost:9092".to_string();
    let topics = vec![cluster_id.clone()];

    let config = KafkaClientConfig::new(cluster_id, bootstrap_servers).with_topics(topics);

    tracing::info!("init stream handler with config: {:?}", config);

    // Using the new routes! macro for cleaner syntax
    let routes = rust_common::routes![
        "/api/v1/login" => |msg| async move {
            tracing::info!("login {:?}", msg);
            Ok(rust_common::kafka::HandlerResult::Response(
                serde_json::json!({
                    "demo": "demo",
                }),
            ))
        },
        "/api/v1/register" => |msg| async move {
            tracing::info!("register {:?}", msg);
            Ok(rust_common::kafka::HandlerResult::Response(
                serde_json::json!({
                    "token": "456",
                }),
            ))
        }
    ];

    let background_task = rust_common::kafka::StreamHandler::new(config, routes)?
        .start()
        .await?;

    tracing::info!("stream handler started");

    signal::ctrl_c().await?;
    tracing::info!("Shutting down...");
    background_task.abort();

    Ok(())
}
