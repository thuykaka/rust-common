// use anyhow::Context;
// use rust_common::kafka::{config::KafkaConfig, request_sender::RequestSender};
// use rust_common::logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //     logger::init_with_default().context("Failed to initialize logger")?;

    //     let kafka_config = KafkaConfig {
    //         client_id: None,
    //         cluster_id: "aaa".to_string(),
    //         bootstrap_servers: "localhost:9092".to_string(),
    //         topics: None,
    //     };

    //     let request_sender = RequestSender::new(kafka_config)?;

    //     request_sender.start_processing().await?;

    Ok(())
}
