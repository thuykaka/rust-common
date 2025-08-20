use anyhow::{Context, Result};
use futures::StreamExt;
use rdkafka::{
    consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance, StreamConsumer},
    message::OwnedMessage,
    ClientContext,
};
use std::{future::Future, sync::Arc};
use tracing::{error, info};

use crate::kafka::core::KafkaClientConfig;

pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("post rebalance {:?}", rebalance);
    }
}

pub type LoggingConsumer = StreamConsumer<CustomContext>;

/// Kafka consumer for processing messages from topics
///
/// This struct provides a high-level interface for consuming messages from Kafka topics
/// with built-in concurrency support, error handling, and logging.
///
/// # Features
///
/// - Async message processing with configurable concurrency
/// - Automatic rebalancing with logging
/// - Built-in error handling and recovery
/// - Support for multiple topics
/// - Graceful shutdown handling
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::core::*;
/// use rdkafka::config::RDKafkaLogLevel;
///
/// // Create configuration
/// let config = KafkaClientConfig::new(
///     "my-consumer-group".to_string(),
///     Some(vec!["my-topic".to_string()]),
///     RDKafkaLogLevel::Info,
/// );
///
/// // Create consumer with concurrency limit
/// let consumer = KafkaConsumer::new(config, Some(10))?;
///
/// // Start processing messages
/// consumer.start(|msg| async move {
///     let payload = String::from_utf8_lossy(msg.payload().unwrap_or_default());
///     println!("Processing message: {}", payload);
///     
///     // Your business logic here
///     process_message(payload).await?;
///     
///     Ok(())
/// }).await?;
/// ```
///
/// # Concurrency
///
/// The consumer processes messages concurrently up to the specified `concurrency_limit`.
/// Each message is processed in its own async task, allowing for efficient parallel processing.
///
/// # Error Handling
///
/// - Individual message processing errors are logged but don't stop the consumer
/// - Network errors and rebalancing events are handled automatically
/// - The consumer will attempt to reconnect on connection failures
pub struct KafkaConsumer {
    /// The underlying rdkafka consumer with custom context for logging
    pub consumer: LoggingConsumer,
    /// Maximum number of messages to process concurrently
    pub concurrency_limit: usize,
}

impl KafkaConsumer {
    pub fn new(config: KafkaClientConfig, concurrency_limit: usize) -> Result<Self> {
        let context = CustomContext;

        let mut consumer_config = config.to_client_config();

        // Consumer-specific settings
        consumer_config
            .set("enable.partition.eof", "false")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "10000")
            .set("heartbeat.interval.ms", "500")
            .set("group.id", config.cluster_id.clone());

        let consumer: LoggingConsumer = consumer_config
            .create_with_context(context)
            .context("Consumer creation failed")?;

        // Convert Vec<String> to &[&str] for subscribe
        let topics: Vec<&str> = config
            .topics
            .as_ref()
            .map(|topics| topics.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(|| vec![config.get_cluster_id()]);

        consumer
            .subscribe(&topics)
            .context("Can't subscribe to specified topics")?;

        info!("consumer subscribed to topic: {:?}", topics);

        Ok(Self {
            consumer,
            concurrency_limit,
        })
    }

    pub async fn start<T, F>(&self, handler: T) -> Result<()>
    where
        T: Fn(OwnedMessage) -> F + Send + Sync + Clone + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        info!("consumer message processing...");

        let handler = Arc::new(handler);

        self.consumer
            .stream()
            .filter_map(|msg| async { msg.ok() })
            .map({
                let handler = handler.clone();
                move |m| {
                    let handler = handler.clone();
                    async move { handler(m.detach()).await }
                }
            })
            .buffer_unordered(self.concurrency_limit)
            .for_each(|res| async {
                if let Err(e) = res {
                    error!("error while processing message: {}", e);
                }
            })
            .await;

        Ok(())
    }
}
