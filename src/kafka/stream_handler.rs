//! # Kafka Stream Handler
//!
//! This module provides a high-performance, configurable Kafka message stream handler
//! with support for custom message processing, response routing, and controlled concurrency.
//!
//! ## Features
//!
//! - **Controlled Concurrency**: Uses `buffer_unordered` to limit concurrent message processing
//! - **Custom Handlers**: Register async functions to process messages for specific URI patterns
//! - **Response Routing**: Automatic routing of responses back to message originators
//! - **Error Handling**: Graceful error handling with comprehensive logging
//! - **Resource Management**: Efficient memory usage with Arc-based sharing
//!
//! ## Quick Start
//!
//! ```rust
//! use rust_common::kafka::{KafkaConfig, StreamHandler, HandlerResult};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = KafkaConfig {
//!         bootstrap_servers: "localhost:9092".to_string(),
//!         cluster_id: "my-cluster".to_string(),
//!         topics: vec!["my-topic".to_string()],
//!     };
//!
//!     let mut handler = StreamHandler::new(config)?;
//!
//!     // Register a message handler
//!     handler.register("/api/users", |msg| async move {
//!         // Process the message
//!         Ok(HandlerResult::Acknowledge)
//!     })?;
//!
//!     // Start processing
//!     handler.start_processing().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The stream handler uses a functional programming approach with:
//! - **filter_map**: Filters out invalid messages
//! - **map**: Transforms messages into processing tasks
//! - **buffer_unordered**: Controls concurrency with a configurable buffer
//! - **for_each**: Handles processing results
//!
//! This design ensures efficient resource usage while maintaining high throughput
//! and providing excellent error handling capabilities.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};

use crate::kafka::{
    utils::{create_message, extract_payload},
    Error, KafkaConfig, MessageType, ParsedMessage,
};
use anyhow::{Context, Result};
use futures::StreamExt;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance, StreamConsumer},
    message::OwnedMessage,
    producer::{FutureProducer, FutureRecord},
    ClientConfig, ClientContext, Message,
};
use tracing::{info, warn};

use crate::kafka::extensions::MessageLatency;

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("post rebalance {:?}", rebalance);
    }
}

type LoggingConsumer = StreamConsumer<CustomContext>;

/// Result of processing a Kafka message.
///
/// This enum defines the possible outcomes when a message handler processes a message.
/// It allows handlers to either send a response back to the sender or simply acknowledge
/// that the message was processed successfully.
#[derive(Debug)]
pub enum HandlerResult {
    /// Send a response back to the sender.
    ///
    /// Use this variant when you need to send data back to the message originator.
    /// The response will be automatically routed to the appropriate topic and URI.
    ///
    /// # Example
    ///
    /// ```rust
    /// Ok(HandlerResult::Response(serde_json::json!({
    ///     "status": "success",
    ///     "data": { "user_id": 123 }
    /// })))
    /// ```
    Response(serde_json::Value),

    /// Acknowledge the message was processed successfully without sending a response.
    ///
    /// Use this variant when you've successfully processed the message but don't need
    /// to send any data back to the sender. This is useful for fire-and-forget operations
    /// or when the processing is purely internal.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Process the message and acknowledge
    /// process_user_registration(&msg).await?;
    /// Ok(HandlerResult::Acknowledge)
    /// ```
    Acknowledge,
}

/// Type alias for a message handler function.
///
/// A message handler is an async function that processes a Kafka message and returns
/// a `HandlerResult`. The function receives a reference to the parsed message and
/// can perform any necessary processing logic.
///
/// # Example
///
/// ```rust
/// let handler: MessageHandler = Arc::new(|msg: &ParsedMessage| {
///     Box::pin(async move {
///         // Process the message
///         match msg.uri.as_str() {
///             "/api/users" => {
///                 let user_data = process_user_data(&msg.payload)?;
///                 Ok(HandlerResult::Response(serde_json::to_value(user_data)?))
///             }
///             _ => Ok(HandlerResult::Acknowledge)
///         }
///     })
/// });
/// ```
pub type MessageHandler = Arc<
    dyn Fn(
            &ParsedMessage,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<HandlerResult, Error>> + Send>,
        > + Send
        + Sync,
>;

/// A high-performance Kafka message stream handler with configurable concurrency.
///
/// This struct provides an efficient way to process Kafka messages with:
/// - Configurable concurrency limits using `buffer_unordered`
/// - Automatic error handling and recovery
/// - Support for custom message handlers
/// - Response routing capabilities
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::{KafkaConfig, StreamHandler};
///
/// let config = KafkaConfig {
///     bootstrap_servers: "localhost:9092".to_string(),
///     cluster_id: "my-cluster".to_string(),
///     topics: vec!["my-topic".to_string()],
/// };
///
/// let mut handler = StreamHandler::new(config)?;
///
/// // Register a message handler
/// handler.register("/api/users", |msg| async move {
///     // Process the message
///     Ok(HandlerResult::Acknowledge)
/// })?;
///
/// // Start processing with default concurrency limit (100)
/// handler.start_processing().await?;
/// ```
pub struct StreamHandler {
    config: KafkaConfig,
    consumer: LoggingConsumer,
    producer: Arc<FutureProducer>,
    routes: Arc<RwLock<HashMap<String, MessageHandler>>>,
    concurrency_limit: usize,
}

impl StreamHandler {
    /// Creates a new `StreamHandler` with default concurrency limit (100).
    ///
    /// This is the recommended way to create a `StreamHandler` for most use cases.
    /// The default concurrency limit of 100 provides a good balance between
    /// performance and resource usage.
    ///
    /// # Arguments
    ///
    /// * `config` - Kafka configuration including bootstrap servers, cluster ID, and topics
    ///
    /// # Returns
    ///
    /// Returns `Ok(StreamHandler)` on success, or `Err` if Kafka consumer/producer creation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = KafkaConfig {
    ///     bootstrap_servers: "localhost:9092".to_string(),
    ///     cluster_id: "my-cluster".to_string(),
    ///     topics: vec!["my-topic".to_string()],
    /// };
    ///
    /// let handler = StreamHandler::new(config)?;
    /// ```
    pub fn new(config: KafkaConfig) -> anyhow::Result<Self> {
        Self::with_concurrency_limit(config, 100)
    }

    /// Creates a new `StreamHandler` with a custom concurrency limit.
    ///
    /// Use this constructor when you need fine-grained control over the number of
    /// messages processed concurrently. Higher limits may improve throughput but
    /// also increase memory usage and CPU load.
    ///
    /// # Arguments
    ///
    /// * `config` - Kafka configuration including bootstrap servers, cluster ID, and topics
    /// * `concurrency_limit` - Maximum number of messages to process concurrently
    ///
    /// # Returns
    ///
    /// Returns `Ok(StreamHandler)` on success, or `Err` if Kafka consumer/producer creation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = KafkaConfig { /* ... */ };
    ///
    /// // Create handler with low concurrency for resource-constrained environments
    /// let handler = StreamHandler::with_concurrency_limit(config, 10)?;
    ///
    /// // Create handler with high concurrency for high-throughput scenarios
    /// let handler = StreamHandler::with_concurrency_limit(config, 500)?;
    /// ```
    pub fn with_concurrency_limit(
        config: KafkaConfig,
        concurrency_limit: usize,
    ) -> anyhow::Result<Self> {
        let context = CustomContext;

        let mut consumer_config = ClientConfig::new();

        consumer_config
            .set("group.id", &config.cluster_id)
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("allow.auto.create.topics", "true")
            .set("enable.partition.eof", "false")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "10000")
            .set("heartbeat.interval.ms", "500")
            .set_log_level(RDKafkaLogLevel::Debug);

        let consumer: LoggingConsumer = consumer_config
            .create_with_context(context)
            .context("Consumer creation failed")?;

        // Convert Vec<String> to &[&str] for subscribe
        let topics: Vec<&str> = config.topics.iter().map(|s| s.as_str()).collect();

        consumer
            .subscribe(&topics)
            .context("Can't subscribe to specified topics")?;

        let mut producer_config = ClientConfig::new();

        producer_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .set("allow.auto.create.topics", "true")
            .set_log_level(RDKafkaLogLevel::Debug);

        let producer: FutureProducer = producer_config
            .create()
            .context("Producer creation failed")?;

        Ok(Self {
            config,
            consumer,
            producer: Arc::new(producer),
            routes: Arc::new(RwLock::new(HashMap::new())),
            concurrency_limit,
        })
    }

    /// Registers a message handler for a specific URI pattern.
    ///
    /// This method allows you to register custom async functions that will be called
    /// when messages with matching URIs are received. The handler function receives
    /// the parsed message and can return either a response or an acknowledgment.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI pattern to match incoming messages (e.g., "/api/users")
    /// * `f` - An async function that processes the message and returns a `HandlerResult`
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err` if the handler cannot be registered.
    ///
    /// # Example
    ///
    /// ```rust
    /// handler.register("/api/users", |msg| async move {
    ///     // Extract user data from message
    ///     let user_data = serde_json::from_value::<UserData>(msg.payload.clone())?;
    ///     
    ///     // Process the user data
    ///     let result = process_user_registration(&user_data).await?;
    ///     
    ///     // Return response
    ///     Ok(HandlerResult::Response(serde_json::json!({
    ///         "status": "success",
    ///         "user_id": result.user_id
    ///     })))
    /// })?;
    ///
    /// // Register a fire-and-forget handler
    /// handler.register("/api/logs", |msg| async move {
    ///     log_message(&msg.payload).await?;
    ///     Ok(HandlerResult::Acknowledge)
    /// })?;
    /// ```
    pub fn register<F, Fut>(&mut self, uri: &str, f: F) -> Result<(), Error>
    where
        F: Fn(ParsedMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<HandlerResult, Error>> + Send + 'static,
    {
        let handler = Arc::new(move |msg: &ParsedMessage| {
            let fut = f(msg.clone());
            Box::pin(fut)
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<HandlerResult, Error>> + Send>,
                >
        });

        self.routes
            .write()
            .map_err(|e| Error::InternalServerError(format!("Failed to acquire lock: {}", e)))?
            .insert(uri.to_string(), handler);

        info!("registered handler for uri: {}", uri);

        Ok(())
    }

    /// Returns a reference to the Kafka configuration.
    ///
    /// This method provides access to the configuration used to create the handler,
    /// including bootstrap servers, cluster ID, and subscribed topics.
    ///
    /// # Returns
    ///
    /// A reference to the `KafkaConfig` used by this handler.
    pub fn get_config(&self) -> &KafkaConfig {
        &self.config
    }

    /// Returns the current concurrency limit.
    ///
    /// The concurrency limit determines the maximum number of messages that can be
    /// processed simultaneously. This value is used by `buffer_unordered` to control
    /// resource usage.
    ///
    /// # Returns
    ///
    /// The current concurrency limit as a `usize`.
    pub fn get_concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }

    /// Sets the concurrency limit for message processing.
    ///
    /// This method allows you to dynamically adjust the concurrency limit without
    /// recreating the handler. Changes take effect immediately for new messages.
    ///
    /// # Arguments
    ///
    /// * `limit` - The new concurrency limit (must be > 0)
    ///
    /// # Example
    ///
    /// ```rust
    /// // Reduce concurrency for resource-constrained periods
    /// handler.set_concurrency_limit(10);
    ///
    /// // Increase concurrency for high-load scenarios
    /// handler.set_concurrency_limit(200);
    /// ```
    pub fn set_concurrency_limit(&mut self, limit: usize) {
        self.concurrency_limit = limit;
    }

    /// Checks if a handler is registered for the given URI.
    ///
    /// This method is useful for debugging or monitoring which URIs have registered
    /// handlers. It can also be used to implement dynamic handler registration logic.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to check for registered handlers
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if a handler is registered for the URI, `Ok(false)` if not,
    /// or `Err` if the check cannot be performed.
    ///
    /// # Example
    ///
    /// ```rust
    /// if handler.has_handler("/api/users")? {
    ///     println!("Handler for /api/users is registered");
    /// } else {
    ///     println!("No handler for /api/users");
    /// }
    /// ```
    pub fn has_handler(&self, uri: &str) -> Result<bool, Error> {
        self.routes
            .read()
            .map_err(|e| Error::InternalServerError(format!("Failed to acquire read lock: {}", e)))
            .map(|guard| guard.contains_key(uri))
    }

    /// Returns a list of all registered URI patterns.
    ///
    /// This method is useful for debugging, monitoring, or implementing dynamic
    /// handler management. It returns all URIs that currently have registered handlers.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<String>)` containing all registered URIs, or `Err` if the
    /// operation cannot be completed.
    ///
    /// # Example
    ///
    /// ```rust
    /// let uris = handler.get_registered_uris()?;
    /// println!("Registered handlers:");
    /// for uri in uris {
    ///     println!("  - {}", uri);
    /// }
    /// ```
    pub fn get_registered_uris(&self) -> Result<Vec<String>, Error> {
        self.routes
            .read()
            .map_err(|e| Error::InternalServerError(format!("Failed to acquire read lock: {}", e)))
            .map(|guard| guard.keys().cloned().collect())
    }

    fn get_handler(
        routes: &Arc<RwLock<HashMap<String, MessageHandler>>>,
        uri: &str,
    ) -> Result<Option<MessageHandler>, Error> {
        routes
            .read()
            .map_err(|e| Error::InternalServerError(format!("Failed to acquire read lock: {}", e)))
            .map(|guard| guard.get(uri).cloned())
    }

    /// Starts processing Kafka messages with controlled concurrency.
    ///
    /// This method begins consuming messages from the configured Kafka topics and
    /// processes them using the registered handlers. The processing is controlled
    /// by the concurrency limit set during construction, ensuring optimal resource usage.
    ///
    /// The method uses `buffer_unordered` to limit the number of messages processed
    /// concurrently, preventing resource exhaustion while maintaining high throughput.
    ///
    /// # Behavior
    ///
    /// - Messages are consumed from Kafka topics in real-time
    /// - Each message is processed by the appropriate registered handler
    /// - Processing errors are logged but don't stop the stream
    /// - The method runs indefinitely until the stream is interrupted
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the stream completes normally, or `Err` if there's
    /// a critical error that prevents message processing.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Start processing (this will run indefinitely)
    /// handler.start_processing().await?;
    ///
    /// // In a real application, you might want to handle shutdown gracefully:
    /// tokio::select! {
    ///     _ = handler.start_processing() => {
    ///         println!("Processing completed");
    ///     }
    ///     _ = tokio::signal::ctrl_c() => {
    ///         println!("Shutting down...");
    ///     }
    /// }
    /// ```
    pub async fn start_processing(&self) -> anyhow::Result<()> {
        self.consumer
            .stream()
            .filter_map(|msg| async { msg.ok() })
            .map(|m| {
                let producer = Arc::clone(&self.producer);
                let routes = Arc::clone(&self.routes);
                async move { Self::process_message(producer, m.detach(), routes).await }
            })
            .buffer_unordered(self.concurrency_limit)
            .for_each(|res| async {
                if let Err(e) = res {
                    warn!("Error processing message: {}", e);
                }
            })
            .await;

        Ok(())
    }

    async fn send_response(
        producer: Arc<FutureProducer>,
        message_id: String,
        transaction_id: String,
        topic: String,
        uri: String,
        data: serde_json::Value,
    ) -> Result<(), Error> {
        let send_message = create_message(
            None,
            message_id,
            transaction_id,
            topic,
            uri,
            data,
            Some(MessageType::Response),
            None,
        );

        let payload = serde_json::to_string(&send_message.message).map_err(|e| {
            Error::InternalServerError(format!("Failed to serialize response message: {}", e))
        })?;

        let produce_future = producer.as_ref().send(
            FutureRecord::<String, String>::to(&send_message.topic).payload(&payload),
            std::time::Duration::from_secs(5),
        );

        match produce_future.await {
            Ok(_) => {
                info!(
                    "sent message: {:?} to topic: {} success",
                    send_message.message, send_message.topic
                );
            }
            Err((e, _)) => {
                warn!(
                    "send message: {:?} to topic: {} failed: {}",
                    send_message.message, send_message.topic, e
                );
                return Err(Error::InternalServerError(format!(
                    "failed to send response: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    async fn handle_not_found_response(
        producer: Arc<FutureProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
    ) -> Result<(), Error> {
        warn!("no handler found for uri: {}", parsed_message.uri);

        let duration = start_time.elapsed().as_millis();

        info!(
            "1. handler request {} - {} took: {}s",
            parsed_message.uri, parsed_message.transaction_id, duration
        );

        if parsed_message.should_response() {
            let response_destination = parsed_message.response_destination.as_ref().unwrap(); // already checked in should_response

            Self::send_response(
                producer,
                parsed_message.message_id.clone(),
                parsed_message.transaction_id.clone(),
                response_destination.topic.clone(),
                response_destination.uri.clone(),
                serde_json::to_value(Error::UriNotFound(parsed_message.uri.clone()).to_response())
                    .map_err(|e| {
                        Error::InternalServerError(format!(
                            "Failed to serialize not found response: {}",
                            e
                        ))
                    })?,
            )
            .await?;
        }

        Ok(())
    }

    async fn handle_response_ok(
        producer: Arc<FutureProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
        response: serde_json::Value,
    ) -> Result<(), Error> {
        if parsed_message.should_response() {
            let response_destination = parsed_message.response_destination.as_ref().unwrap();

            Self::send_response(
                producer,
                parsed_message.message_id.clone(),
                parsed_message.transaction_id.clone(),
                response_destination.topic.clone(),
                response_destination.uri.clone(),
                serde_json::json!({
                    "data": response
                }),
            )
            .await?;
        }

        let duration = start_time.elapsed().as_millis();
        info!(
            "4. handle request {} - {} took: {}ms",
            parsed_message.uri, parsed_message.transaction_id, duration
        );

        Ok(())
    }

    async fn handle_response_error(
        producer: Arc<FutureProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
        error: Error,
    ) -> Result<(), Error> {
        if parsed_message.should_response() {
            let response_destination = parsed_message.response_destination.as_ref().unwrap();

            Self::send_response(
                producer,
                parsed_message.message_id.clone(),
                parsed_message.transaction_id.clone(),
                response_destination.topic.clone(),
                response_destination.uri.clone(),
                serde_json::to_value(error.to_response()).map_err(|e| {
                    Error::InternalServerError(format!("Failed to serialize error response: {}", e))
                })?,
            )
            .await?;
        }

        let duration = start_time.elapsed().as_millis();
        info!(
            "3. handle request {} - {} took: {}ms",
            parsed_message.uri, parsed_message.transaction_id, duration
        );

        Ok(())
    }

    async fn process_message(
        producer: Arc<FutureProducer>,
        message: OwnedMessage,
        routes: Arc<RwLock<HashMap<String, MessageHandler>>>,
    ) -> anyhow::Result<()> {
        let message_value =
            extract_payload(&message).ok_or_else(|| anyhow::anyhow!("message has no payload"))?;

        let latency = message.get_latency();
        info!(
            "received message: '{}' from topic {}, latency: {}ms",
            message_value,
            message.topic(),
            latency,
        );

        let start_time = Instant::now();

        let parsed_message = ParsedMessage::parse_from_string(&message_value)
            .context("failed to parse message from Kafka payload")?;

        let handler = Self::get_handler(&routes, &parsed_message.uri)?;

        if let Some(handler) = handler {
            match handler(&parsed_message).await {
                Err(e) => {
                    warn!(
                        "error while handler reques: {} - {}: {}",
                        parsed_message.uri.clone(),
                        parsed_message.transaction_id.clone(),
                        e
                    );

                    Self::handle_response_error(producer, &parsed_message, start_time, e).await?;
                }
                Ok(result) => match result {
                    HandlerResult::Response(response) => {
                        Self::handle_response_ok(producer, &parsed_message, start_time, response)
                            .await?;
                    }
                    HandlerResult::Acknowledge => {
                        let duration = start_time.elapsed().as_millis();
                        info!(
                            "2. acknowledge request {} - {} took: {}ms",
                            parsed_message.uri, parsed_message.transaction_id, duration
                        );
                    }
                },
            }
        } else {
            Self::handle_not_found_response(producer, &parsed_message, start_time).await?;
        }

        Ok(())
    }
}
