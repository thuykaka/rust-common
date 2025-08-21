use anyhow::Context;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::sync::Arc;
use tracing::{error, info};

use crate::kafka::core::{KafkaClientConfig, KafkaError};

/// KafkaProducer is responsible for sending messages to Kafka topics asynchronously.
/// It wraps the rdkafka FutureProducer for thread-safe operations.
#[derive(Clone)]
pub struct KafkaProducer {
    /// The underlying rdkafka producer wrapped in Arc for thread safety
    pub producer: Arc<FutureProducer>,
}

impl KafkaProducer {
    /// Creates a new KafkaProducer with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - KafkaClientConfig containing the necessary settings for the producer.
    ///
    /// # Returns
    ///
    /// * `anyhow::Result<Self>` - Returns a KafkaProducer instance or an error if creation fails.
    pub fn new(config: KafkaClientConfig) -> anyhow::Result<Self> {
        let mut producer_config = config.to_client_config();

        producer_config.set("acks", "0");
        producer_config.set("transaction.timeout.ms", "60000");
        producer_config.set("message.send.max.retries", "10");

        let producer: FutureProducer = producer_config
            .create()
            .context("Producer creation failed")?;

        Ok(Self {
            producer: Arc::new(producer),
        })
    }

    /// Sends a message to the specified Kafka topic.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be sent, which must implement `serde::Serialize` and `std::fmt::Debug`.
    /// * `topic` - The topic to which the message will be sent.
    ///
    /// # Returns
    ///
    /// * `anyhow::Result<(), KafkaError>` - Returns Ok if the message is sent successfully, or a KafkaError if it fails.
    pub async fn send<T>(&self, message: T, topic: &str) -> anyhow::Result<(), KafkaError>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        let payload = serde_json::to_string(&message).map_err(|e| {
            KafkaError::InternalServerError(format!("Failed to serialize response message: {}", e))
        })?;

        let _ = self
            .producer
            .send(
                FutureRecord::<String, String>::to(&topic).payload(&payload),
                std::time::Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| {
                error!(
                    "sent message: {:?} to topic: {} failed: {}",
                    message, topic, e
                );
                KafkaError::InternalServerError(format!("Failed to send message to Kafka: {}", e))
            })?;

        info!("sent message: {:?} to topic: {} success", message, topic);

        Ok(())
    }
}
