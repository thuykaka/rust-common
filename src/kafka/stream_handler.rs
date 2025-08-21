use std::{sync::Arc, time::Instant};

use crate::kafka::{
    utils::utils::{create_message, extract_payload},
    HandlerResult, KafkaClientConfig, KafkaConsumer, KafkaError, KafkaProducer, MessageType,
    ParsedMessage, RouteRegistry,
};
use anyhow::{anyhow, Context, Result};
use rdkafka::{message::OwnedMessage, Message};
use tracing::{error, info, warn};

use crate::kafka::extensions::MessageLatency;

/// StreamHandler is responsible for processing Kafka messages using a route-based system.
/// It manages the consumer and producer, and handles message routing and response.
pub struct StreamHandler {
    config: KafkaClientConfig,
    consumer: KafkaConsumer,
    producer: Arc<KafkaProducer>,
    route_registry: RouteRegistry,
}

impl StreamHandler {
    const DEFAULT_CONCURRENCY_LIMIT: usize = 100;

    /// Creates a new StreamHandler with the given configuration and route registry.
    ///
    /// # Arguments
    ///
    /// * `config` - KafkaClientConfig containing the necessary settings.
    /// * `route_registry` - The registry of routes for message handling.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a StreamHandler instance or an error if creation fails.
    pub fn new(config: KafkaClientConfig, route_registry: RouteRegistry) -> Result<Self> {
        Self::with_concurrency_limit(config, route_registry, Self::DEFAULT_CONCURRENCY_LIMIT)
    }

    /// Creates a new StreamHandler with a specified concurrency limit.
    ///
    /// # Arguments
    ///
    /// * `config` - KafkaClientConfig containing the necessary settings.
    /// * `route_registry` - The registry of routes for message handling.
    /// * `concurrency_limit` - The maximum number of messages to process concurrently.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a StreamHandler instance or an error if creation fails.
    pub fn with_concurrency_limit(
        config: KafkaClientConfig,
        route_registry: RouteRegistry,
        concurrency_limit: usize,
    ) -> Result<Self> {
        let consumer = KafkaConsumer::new(config.clone(), concurrency_limit)
            .context("failed to create Kafka consumer")?;

        let producer =
            KafkaProducer::new(config.clone()).context("failed to create Kafka producer")?;

        Ok(Self {
            config,
            consumer,
            producer: Arc::new(producer),
            route_registry,
        })
    }

    /// Retrieves the KafkaClientConfig associated with the StreamHandler.
    ///
    /// # Returns
    ///
    /// * `&KafkaClientConfig` - A reference to the configuration.
    pub fn get_config(&self) -> &KafkaClientConfig {
        &self.config
    }

    /// Starts the StreamHandler to process messages using the registered routes.
    ///
    /// # Returns
    ///
    /// * `Result<tokio::task::JoinHandle<()>>` - Returns a handle to the spawned task or an error if it fails.
    pub async fn start(&self) -> Result<tokio::task::JoinHandle<()>> {
        let route_registry = self.route_registry.clone();
        let producer = self.producer.clone();
        let source_id = self.config.cluster_id.clone();

        let consumer_task =
            self.consumer
                .start(move |message| {
                    let route_registry = route_registry.clone();
                    let producer = producer.clone();
                    let source_id = source_id.clone();
                    async move {
                        Self::handle_message(message, source_id, route_registry, producer).await
                    }
                })
                .await?;

        Ok(consumer_task)
    }

    /// Sends a response message using the producer.
    ///
    /// # Arguments
    ///
    /// * `producer` - The KafkaProducer to use for sending the message.
    /// * `source_id` - The source identifier for the message.
    /// * `message_id` - The unique message identifier.
    /// * `transaction_id` - The transaction identifier for tracking.
    /// * `topic` - The topic to which the message will be sent.
    /// * `uri` - The URI for message routing.
    /// * `data` - The data payload of the message.
    ///
    /// # Returns
    ///
    /// * `Result<(), KafkaError>` - Returns Ok if the message is sent successfully, or a KafkaError if it fails.
    async fn send_response(
        producer: Arc<KafkaProducer>,
        source_id: String,
        message_id: String,
        transaction_id: String,
        topic: String,
        uri: String,
        data: serde_json::Value,
    ) -> Result<(), KafkaError> {
        let send_message = create_message(
            source_id,
            message_id,
            transaction_id,
            topic,
            uri,
            data,
            Some(MessageType::Response),
            None,
        );

        producer
            .send(send_message.message, &send_message.topic)
            .await
            .map_err(|e| {
                KafkaError::InternalServerError(format!("failed to send response: {}", e))
            })?;

        Ok(())
    }

    /// Handles the response for a parsed message.
    ///
    /// # Arguments
    ///
    /// * `producer` - The KafkaProducer to use for sending the response.
    /// * `source_id` - The source identifier for the message.
    /// * `parsed_message` - The parsed message to handle.
    /// * `start_time` - The time when the message processing started.
    /// * `response_data` - The data to include in the response.
    /// * `log_prefix` - A prefix for logging purposes.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns Ok if the response is handled successfully, or an error if it fails.
    async fn handle_response(
        producer: Arc<KafkaProducer>,
        source_id: String,
        parsed_message: &ParsedMessage,
        start_time: Instant,
        response_data: serde_json::Value,
        log_prefix: &str,
    ) -> Result<()> {
        if parsed_message.should_response() {
            let response_destination = parsed_message.get_response_destination().unwrap();

            Self::send_response(
                producer,
                source_id,
                parsed_message.message_id.clone(),
                parsed_message.transaction_id.clone(),
                response_destination.topic.clone(),
                response_destination.uri.clone(),
                response_data,
            )
            .await?;
        }

        let duration = start_time.elapsed().as_millis();
        info!(
            "{} handle request {} - {} took: {}ms",
            log_prefix, parsed_message.uri, parsed_message.transaction_id, duration
        );

        Ok(())
    }

    /// Sends a response indicating that the requested URI was not found.
    ///
    /// # Arguments
    ///
    /// * `producer` - The KafkaProducer to use for sending the response.
    /// * `parsed_message` - The parsed message to handle.
    /// * `source_id` - The source identifier for the message.
    /// * `start_time` - The time when the message processing started.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns Ok if the response is sent successfully, or an error if it fails.
    async fn send_not_found_uri_response(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        source_id: String,
        start_time: Instant,
    ) -> Result<()> {
        warn!("no handler found for uri: {}", parsed_message.uri);

        Self::handle_response(
            producer,
            source_id,
            parsed_message,
            start_time,
            KafkaError::UriNotFound(parsed_message.uri.clone()).to_response_value(),
            "1.",
        )
        .await
    }

    /// Handles an error response for a parsed message.
    ///
    /// # Arguments
    ///
    /// * `producer` - The KafkaProducer to use for sending the response.
    /// * `parsed_message` - The parsed message to handle.
    /// * `source_id` - The source identifier for the message.
    /// * `start_time` - The time when the message processing started.
    /// * `error` - The error to include in the response.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns Ok if the response is handled successfully, or an error if it fails.
    async fn handle_response_error(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        source_id: String,
        start_time: Instant,
        error: KafkaError,
    ) -> Result<()> {
        Self::handle_response(
            producer,
            source_id,
            parsed_message,
            start_time,
            error.to_response_value(),
            "3.",
        )
        .await
    }

    /// Handles a successful response for a parsed message.
    ///
    /// # Arguments
    ///
    /// * `producer` - The KafkaProducer to use for sending the response.
    /// * `parsed_message` - The parsed message to handle.
    /// * `source_id` - The source identifier for the message.
    /// * `start_time` - The time when the message processing started.
    /// * `response` - The response data to include.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns Ok if the response is handled successfully, or an error if it fails.
    async fn handle_response_ok(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        source_id: String,
        start_time: Instant,
        response: serde_json::Value,
    ) -> Result<()> {
        Self::handle_response(
            producer,
            source_id,
            parsed_message,
            start_time,
            serde_json::json!({
                "data": response
            }),
            "4.",
        )
        .await
    }

    /// Handles an incoming Kafka message by routing it to the appropriate handler.
    ///
    /// # Arguments
    ///
    /// * `message` - The Kafka message to handle.
    /// * `source_id` - The source identifier for the message.
    /// * `route_registry` - The registry of routes for message handling.
    /// * `producer` - The KafkaProducer to use for sending responses.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns Ok if the message is handled successfully, or an error if it fails.
    async fn handle_message(
        message: OwnedMessage,
        source_id: String,
        route_registry: RouteRegistry,
        producer: Arc<KafkaProducer>,
    ) -> Result<()> {
        let start_time = Instant::now();

        let payload = extract_payload(&message).ok_or_else(|| anyhow!("message has no payload"))?;

        let latency = message.get_latency();

        info!(
            "received message: '{}' from topic {}, latency: {}ms",
            payload,
            message.topic(),
            latency,
        );

        let parsed_message = ParsedMessage::parse_from_string(&payload)
            .context("failed to parse message from kafka payload")?;

        let handler = route_registry.get_handler(&parsed_message.uri)?;

        if let Some(handler) = handler {
            match handler(&parsed_message).await {
                Err(e) => {
                    error!(
                        "error handling request {} - {}: {}",
                        parsed_message.uri, parsed_message.transaction_id, e
                    );
                    Self::handle_response_error(
                        producer,
                        &parsed_message,
                        source_id,
                        start_time,
                        e,
                    )
                    .await?;
                }
                Ok(result) => match result {
                    HandlerResult::Acknowledge => {
                        let duration = start_time.elapsed().as_millis();
                        info!(
                            "2. acknowledge request {} - {} (no response) took: {}ms",
                            parsed_message.uri, parsed_message.transaction_id, duration
                        );
                    }
                    HandlerResult::Response(response) => {
                        Self::handle_response_ok(
                            producer,
                            &parsed_message,
                            source_id,
                            start_time,
                            response,
                        )
                        .await?;
                    }
                },
            }
        } else {
            Self::send_not_found_uri_response(producer, &parsed_message, source_id, start_time)
                .await?;
        }

        Ok(())
    }
}
