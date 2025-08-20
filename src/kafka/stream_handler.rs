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

pub struct StreamHandler {
    config: KafkaClientConfig,
    consumer: KafkaConsumer,
    producer: Arc<KafkaProducer>,
    route_registry: RouteRegistry,
}

impl StreamHandler {
    const DEFAULT_CONCURRENCY_LIMIT: usize = 100;

    pub fn new(config: KafkaClientConfig, route_registry: RouteRegistry) -> Result<Self> {
        Self::with_concurrency_limit(config, route_registry, Self::DEFAULT_CONCURRENCY_LIMIT)
    }

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

    pub fn get_config(&self) -> &KafkaClientConfig {
        &self.config
    }

    pub async fn start(&self) -> Result<()> {
        let route_registry = self.route_registry.clone();
        let producer = self.producer.clone();

        self.consumer
            .start(move |message| {
                let route_registry = route_registry.clone();
                let producer = producer.clone();
                async move { Self::handle_message(message, route_registry, producer).await }
            })
            .await
    }

    async fn send_response(
        producer: Arc<KafkaProducer>,
        message_id: String,
        transaction_id: String,
        topic: String,
        uri: String,
        data: serde_json::Value,
    ) -> Result<(), KafkaError> {
        let send_message = create_message(
            None, // todo: add source
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

    async fn handle_response(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
        response_data: serde_json::Value,
        log_prefix: &str,
    ) -> Result<()> {
        if parsed_message.should_response() {
            let response_destination = parsed_message.get_response_destination().unwrap();

            Self::send_response(
                producer,
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

    async fn send_not_found_uri_response(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
    ) -> Result<()> {
        warn!("no handler found for uri: {}", parsed_message.uri);

        Self::handle_response(
            producer,
            parsed_message,
            start_time,
            KafkaError::UriNotFound(parsed_message.uri.clone()).to_response_value(),
            "1.",
        )
        .await
    }

    async fn handle_response_error(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
        error: KafkaError,
    ) -> Result<()> {
        Self::handle_response(
            producer,
            parsed_message,
            start_time,
            error.to_response_value(),
            "3.",
        )
        .await
    }

    async fn handle_response_ok(
        producer: Arc<KafkaProducer>,
        parsed_message: &ParsedMessage,
        start_time: Instant,
        response: serde_json::Value,
    ) -> Result<()> {
        Self::handle_response(
            producer,
            parsed_message,
            start_time,
            serde_json::json!({
                "data": response
            }),
            "4.",
        )
        .await
    }

    async fn handle_message(
        message: OwnedMessage,
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
                    Self::handle_response_error(producer, &parsed_message, start_time, e).await?;
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
                        Self::handle_response_ok(producer, &parsed_message, start_time, response)
                            .await?;
                    }
                },
            }
        } else {
            Self::send_not_found_uri_response(producer, &parsed_message, start_time).await?;
        }

        Ok(())
    }
}
