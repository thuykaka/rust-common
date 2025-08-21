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

/// KafkaConsumer is responsible for consuming messages from Kafka topics asynchronously.
/// It uses a custom context for logging and supports concurrent message processing.
pub struct KafkaConsumer {
    /// The underlying rdkafka consumer with custom context for logging
    pub consumer: Arc<LoggingConsumer>,
    /// Maximum number of messages to process concurrently
    pub concurrency_limit: usize,
}

impl KafkaConsumer {
    /// Creates a new KafkaConsumer with the given configuration and concurrency limit.
    ///
    /// # Arguments
    ///
    /// * `config` - KafkaClientConfig containing the necessary settings for the consumer.
    /// * `concurrency_limit` - The maximum number of messages to process concurrently.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a KafkaConsumer instance or an error if creation fails.
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
            .set("group.id", config.cluster_id.clone())
            .set("fetch.message.max.bytes", "1000000000");

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
            consumer: Arc::new(consumer),
            concurrency_limit,
        })
    }

    /// Starts the consumer to process messages using the provided handler function.
    ///
    /// # Arguments
    ///
    /// * `handler` - A function that processes each message, returning a future.
    ///
    /// # Returns
    ///
    /// * `Result<tokio::task::JoinHandle<()>>` - Returns a handle to the spawned task or an error if it fails.
    pub async fn start<T, F>(&self, handler: T) -> Result<tokio::task::JoinHandle<()>>
    where
        T: Fn(OwnedMessage) -> F + Send + Sync + Clone + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let consumer = self.consumer.clone();
        let handler = Arc::new(handler);
        let handler_for_spawn = handler.clone();
        let concurrency_limit = self.concurrency_limit;

        let consumer_task = tokio::spawn(async move {
            info!("consumer message processing...");

            let _ = tx.send(()); // Signal that consumer is ready to process messages

            consumer
                .stream()
                .for_each_concurrent(concurrency_limit, |res| async {
                    match res {
                        Err(e) => {
                            error!("error while processing message: {}", e);
                        }
                        Ok(m) => {
                            let owned_message = m.detach();
                            let handler = handler_for_spawn.clone();
                            let _ = handler(owned_message).await;
                        }
                    }
                })
                .await;
        });

        // Wait for consumer to be ready
        rx.await?;
        info!("consumer is ready to process messages");

        Ok(consumer_task)
    }
}
