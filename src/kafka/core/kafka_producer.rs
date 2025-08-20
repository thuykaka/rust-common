use anyhow::Context;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::sync::Arc;
use tracing::{error, info};

use crate::kafka::core::{KafkaClientConfig, KafkaError};

/// Kafka producer for sending messages to topics
///
/// This struct provides a high-level interface for sending messages to Kafka topics
/// with built-in serialization, error handling, and logging.
///
/// # Features
///
/// - Async message sending with timeout support
/// - Automatic JSON serialization
/// - Comprehensive error handling
/// - Built-in logging for success/failure events
/// - Thread-safe with Arc-based sharing
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::core::*;
/// use rdkafka::config::RDKafkaLogLevel;
/// use serde::Serialize;
///
/// #[derive(Serialize, Debug)]
/// struct UserEvent {
///     user_id: u64,
///     action: String,
///     timestamp: u64,
/// }
///
/// // Create producer
/// let config = KafkaClientConfig::new(
///     "my-cluster".to_string(),
///     None,
///     RDKafkaLogLevel::Info,
/// );
/// let producer = KafkaProducer::new(config)?;
///
/// // Send message
/// let event = UserEvent {
///     user_id: 123,
///     action: "login".to_string(),
///     timestamp: 1234567890,
/// };
///
/// producer.send(event, "user-events").await?;
/// ```
///
/// # Thread Safety
///
/// `KafkaProducer` is thread-safe and can be shared across multiple threads
/// using `Arc<KafkaProducer>`.

#[derive(Clone)]
pub struct KafkaProducer {
    /// The underlying rdkafka producer wrapped in Arc for thread safety
    pub producer: Arc<FutureProducer>,
}

impl KafkaProducer {
    /// Creates a new Kafka producer with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The Kafka client configuration containing connection settings
    ///
    /// # Returns
    ///
    /// Returns a `Result<KafkaProducer>` where:
    /// - `Ok(producer)` - Successfully created producer
    /// - `Err(e)` - Error during producer creation (e.g., connection issues)
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_common::kafka::core::*;
    /// use rdkafka::config::RDKafkaLogLevel;
    ///
    /// let config = KafkaClientConfig::new(
    ///     "my-cluster".to_string(),
    ///     None,
    ///     RDKafkaLogLevel::Info,
    /// );
    ///
    /// let producer = KafkaProducer::new(config)?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Invalid configuration (e.g., missing bootstrap servers)
    /// - Network connectivity issues
    /// - Kafka broker is unavailable
    pub fn new(config: KafkaClientConfig) -> anyhow::Result<Self> {
        let mut producer_config = config.to_client_config();

        producer_config.set("acks", "0");

        let producer: FutureProducer = producer_config
            .create()
            .context("Producer creation failed")?;

        Ok(Self {
            producer: Arc::new(producer),
        })
    }

    /// Sends a message to the specified Kafka topic
    ///
    /// This method serializes the message to JSON and sends it to the specified topic.
    /// The operation is async and includes built-in error handling and logging.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send. Must implement `Serialize` and `Debug` traits
    /// * `topic` - The target Kafka topic name
    ///
    /// # Type Parameters
    ///
    /// * `T` - The message type. Must implement `serde::Serialize` and `std::fmt::Debug`
    ///
    /// # Returns
    ///
    /// Returns a `Result<()>` where:
    /// - `Ok(())` - Message sent successfully
    /// - `Err(e)` - Error during sending (serialization, network, etc.)
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize, Debug)]
    /// struct OrderEvent {
    ///     order_id: String,
    ///     amount: f64,
    ///     user_id: u64,
    /// }
    ///
    /// let event = OrderEvent {
    ///     order_id: "12345".to_string(),
    ///     amount: 99.99,
    ///     user_id: 456,
    /// };
    ///
    /// producer.send(event, "orders").await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Message serialization fails
    /// - Network connectivity issues
    /// - Kafka broker is unavailable
    /// - Topic doesn't exist (if auto.create.topics is disabled)
    ///
    /// # Logging
    ///
    /// This method logs:
    /// - Success: `info!("sent message: {:?} to topic: {} success", message, topic)`
    /// - Failure: `error!("sent message: {:?} to topic: {} failed: {}", message, topic, e)`
    pub async fn send<T>(&self, message: T, topic: &str) -> anyhow::Result<()>
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
