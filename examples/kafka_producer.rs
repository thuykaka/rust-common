use std::time::Duration;

use anyhow::Context;
use rdkafka::{
    config::RDKafkaLogLevel,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use rust_common::logger::{self};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    info!("Starting Kafka producer demo");

    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", "localhost:9092")
        .set("message.timeout.ms", "5000")
        .set("allow.auto.create.topics", "true")
        .set_log_level(RDKafkaLogLevel::Debug);

    let producer: &FutureProducer = &config.create().context("Producer creation failed")?;

    let mut handles = vec![];

    for i in 0..100_000 {
        let producer = producer.clone();
        let handle = tokio::task::spawn(async move {
            let _ = producer
                .send(
                    FutureRecord::to("topic_name")
                        .payload(&format!("Message {}", i))
                        .key(&format!("Key {}", i)),
                    std::time::Duration::from_secs(0),
                )
                .await;
            println!("Delivery status for message {}", i);
        });
        handles.push(handle);
    }

    // Run all tasks
    let _ = futures::future::join_all(handles).await;

    Ok(())
}
