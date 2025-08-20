use anyhow::Context;
use rust_common::kafka::core::KafkaClientConfig;
use rust_common::logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    let cluster_id = "xxx-yyy".to_string();
    let bootstrap_servers = "localhost:9092".to_string();
    let topics = vec![cluster_id.clone()];

    let config = KafkaClientConfig::new(cluster_id, bootstrap_servers).with_topics(topics);

    tracing::info!("init stream handler with config: {:?}", config);

    let route_registry = rust_common::kafka::RouteRegistry::new()
        .register("/api/v1/login", |msg| async move {
            tracing::info!("login {:?}", msg);
            Ok(rust_common::kafka::HandlerResult::Response(
                serde_json::json!({
                    "token": "123",
                }),
            ))
        })?
        .register("/api/v1/register", |msg| async move {
            tracing::info!("register {:?}", msg);
            Ok(rust_common::kafka::HandlerResult::Response(
                serde_json::json!({
                    "token": "456",
                }),
            ))
        })?;

    let _ = rust_common::kafka::StreamHandler::new(config, route_registry)?
        .start()
        .await?;

    Ok(())
}
