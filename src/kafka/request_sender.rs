use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use rdkafka::message::{Message, OwnedMessage};
use tokio::{select, sync::oneshot::Sender};
use tokio::{
    sync::{oneshot, RwLock},
    time::sleep,
};
use tracing::{error, info, warn};

use crate::kafka::{
    utils::utils::{create_message, extract_payload},
    KafkaClientConfig, KafkaConsumer, KafkaError, KafkaProducer, MessageLatency, MessageType,
    ParsedMessage, ResponseDestination,
};

/// RequestAsyncParams holds the parameters for sending asynchronous requests via Kafka.
#[derive(Debug, Clone)]
pub struct RequestAsyncParams {
    /// The topic to which the request will be sent
    pub topic: String,
    /// The URI associated with the request
    pub uri: String,
    /// Optional transaction ID for tracking
    pub transaction_id: Option<String>,
    /// Unique message ID for the request
    pub message_id: String,
    /// The data payload of the request
    pub data: serde_json::Value,
    /// Optional timeout in seconds for the request
    pub timeout_secs: Option<i64>,
}

impl RequestAsyncParams {
    /// Creates a new instance of RequestAsyncParams with the specified topic, URI, and data.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to which the request will be sent.
    /// * `uri` - The URI associated with the request.
    /// * `message_id` - An optional unique message ID for the request.
    /// * `data` - The data payload of the request.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of RequestAsyncParams.
    pub fn new(
        topic: String,
        uri: String,
        message_id: Option<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            topic,
            uri,
            transaction_id: None,
            message_id: message_id.unwrap_or("".to_string()),
            data,
            timeout_secs: None,
        }
    }

    /// Sets the transaction ID for the request.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The transaction ID to set.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated RequestAsyncParams instance.
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Sets the message ID for the request.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The message ID to set.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated RequestAsyncParams instance.
    pub fn with_message_id(mut self, message_id: String) -> Self {
        self.message_id = message_id;
        self
    }

    /// Sets the timeout in seconds for the request.
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - The timeout in seconds to set.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated RequestAsyncParams instance.
    pub fn with_timeout_secs(mut self, timeout_secs: i64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }
}

struct PendingRequest {
    sender: Sender<ParsedMessage>,
    created_at: Instant,
}

impl PendingRequest {
    pub fn new(sender: Sender<ParsedMessage>) -> Self {
        Self {
            sender,
            created_at: Instant::now(),
        }
    }

    pub fn resolve(self, value: ParsedMessage) -> Result<()> {
        let _ = self.sender.send(value);
        Ok(())
    }
}

/// RequestSender manages the sending of asynchronous requests and handling responses via Kafka.
/// It maintains a registry of pending requests and handles message routing and response.
pub struct RequestSender {
    config: KafkaClientConfig,
    consumer: KafkaConsumer,
    producer: Arc<KafkaProducer>,
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    timeout_secs: i64,
    response_topic: String,
}

impl RequestSender {
    const DEFAULT_CONCURRENCY_LIMIT: usize = 100;
    const DEFAULT_TIMEOUT_SECS: i64 = 600;

    /// Creates a new RequestSender with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - KafkaClientConfig containing the necessary settings.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a RequestSender instance or an error if creation fails.
    pub fn new(config: KafkaClientConfig) -> Result<Self> {
        Self::with_concurrency_limit(
            config,
            Self::DEFAULT_CONCURRENCY_LIMIT,
            Self::DEFAULT_TIMEOUT_SECS,
        )
    }

    /// Creates a new RequestSender with a specified concurrency limit and timeout.
    ///
    /// # Arguments
    ///
    /// * `config` - KafkaClientConfig containing the necessary settings.
    /// * `concurrency_limit` - The maximum number of messages to process concurrently.
    /// * `timeout_secs` - The default timeout in seconds for requests.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a RequestSender instance or an error if creation fails.
    pub fn with_concurrency_limit(
        config: KafkaClientConfig,
        concurrency_limit: usize,
        timeout_secs: i64,
    ) -> Result<Self> {
        let response_topic = format!(
            "{}.{}",
            config.cluster_id, "9a2ece8f-0294-49cf-b2c9-9008417caea5"
        );
        let mut consumer_config = config.clone();
        consumer_config.topics = Some(vec![response_topic.clone()]);

        let consumer = KafkaConsumer::new(consumer_config, concurrency_limit)
            .context("failed to create Kafka consumer")?;

        let producer =
            KafkaProducer::new(config.clone()).context("failed to create Kafka producer")?;

        Ok(Self {
            config,
            consumer,
            producer: Arc::new(producer),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            timeout_secs,
            response_topic,
        })
    }

    /// Retrieves the KafkaClientConfig associated with the RequestSender.
    ///
    /// # Returns
    ///
    /// * `&KafkaClientConfig` - A reference to the configuration.
    pub fn get_config(&self) -> &KafkaClientConfig {
        &self.config
    }

    /// Starts the RequestSender to process incoming messages and handle responses.
    ///
    /// # Returns
    ///
    /// * `Result<tokio::task::JoinHandle<()>>` - Returns a handle to the spawned task or an error if it fails.
    pub async fn start(&self) -> Result<tokio::task::JoinHandle<()>> {
        let timeout_secs = self.timeout_secs;
        let pending_requests = Arc::clone(&self.pending_requests);

        let consumer_task = self
            .consumer
            .start(move |message| {
                let pending_requests = Arc::clone(&pending_requests);

                async move { Self::handle_message(message, pending_requests, timeout_secs).await }
            })
            .await?;

        Ok(consumer_task)
    }

    /// Handles an incoming Kafka message by resolving the corresponding pending request.
    ///
    /// # Arguments
    ///
    /// * `message` - The Kafka message to handle.
    /// * `pending_requests` - The registry of pending requests.
    /// * `timeout_secs` - The timeout in seconds for requests.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns Ok if the message is handled successfully, or an error if it fails.
    async fn handle_message(
        message: OwnedMessage,
        pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
        timeout_secs: i64,
    ) -> Result<()> {
        let payload = extract_payload(&message).ok_or_else(|| anyhow!("message has no payload"))?;

        let latency = message.get_latency();

        info!(
            "received message: '{}' from topic {}, latency: {}ms",
            payload,
            message.topic(),
            latency,
        );

        if message.is_expired(timeout_secs) {
            warn!(
                "ignore this request because it is expired {} _ {}",
                timeout_secs, payload
            );
            return Ok(());
        }

        let parsed_message = ParsedMessage::parse_from_string(&payload)
            .context("failed to parse message from kafka payload")?;

        let mut guard = pending_requests.write().await;

        if let Some(request) = guard.remove(&parsed_message.transaction_id) {
            let duration = Instant::now()
                .duration_since(request.created_at)
                .as_millis();

            info!(
                "request {} took {}ms",
                parsed_message.transaction_id, duration
            );

            let _ = request.resolve(parsed_message);
        } else {
            warn!(
                "ignore this request because it is not found {} (maybe timeout)",
                parsed_message.transaction_id
            );
            return Ok(());
        }

        Ok(())
    }

    /// Sends a base request message to the specified topic and URI.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to which the request will be sent.
    /// * `uri` - The URI associated with the request.
    /// * `transaction_id` - The transaction identifier for tracking.
    /// * `message_id` - The unique message identifier.
    /// * `data` - The data payload of the request.
    ///
    /// # Returns
    ///
    /// * `Result<(), KafkaError>` - Returns Ok if the request is sent successfully, or a KafkaError if it fails.
    async fn send_request_base(
        &self,
        topic: String,
        uri: String,
        transaction_id: String,
        message_id: String,
        data: serde_json::Value,
    ) -> Result<(), KafkaError> {
        let send_message = create_message(
            self.config.cluster_id.clone(),
            message_id,
            transaction_id,
            topic,
            uri,
            data,
            Some(MessageType::Request),
            Some(ResponseDestination {
                topic: self.response_topic.clone(),
                uri: "REQUEST_RESPONSE".to_string(),
            }),
        );

        self.producer
            .send(send_message.message, &send_message.topic)
            .await?;

        Ok(())
    }

    /// Sends an asynchronous request and waits for a response.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the request.
    ///
    /// # Returns
    ///
    /// * `Result<ParsedMessage, KafkaError>` - Returns the parsed response message or a KafkaError if it fails.
    pub async fn send_request_async(
        &self,
        params: RequestAsyncParams,
    ) -> Result<ParsedMessage, KafkaError> {
        let transaction_id = params
            .transaction_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let (tx, rx) = oneshot::channel::<ParsedMessage>();

        {
            let mut guard = self.pending_requests.write().await;
            let pending_request = PendingRequest::new(tx);
            guard.insert(transaction_id.clone(), pending_request);
        }

        self.send_request_base(
            params.topic,
            params.uri,
            transaction_id.clone(),
            params.message_id,
            params.data,
        )
        .await?;

        select! {
            res = rx => {
                self.pending_requests.write().await.remove(&transaction_id);
                match res {
                    Ok(response) => Ok(response),
                    Err(e) => Err(KafkaError::InternalServerError(format!("channel closed unexpectedly: {}", e))),
                }
            }
            _ = sleep(Duration::from_secs(params.timeout_secs.unwrap_or(Self::DEFAULT_TIMEOUT_SECS) as u64)) => {
                error!("request {} timeout", transaction_id);
                self.pending_requests.write().await.remove(&transaction_id);
                Err(KafkaError::TimeoutError(format!("request {} timeout", transaction_id)))
            }
        }
    }

    /// Sends a request and waits for an acknowledgment.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the request.
    ///
    /// # Returns
    ///
    /// * `Result<(), KafkaError>` - Returns Ok if the request is acknowledged, or a KafkaError if it fails.
    pub async fn send_request_acknowledge(&self, _: RequestAsyncParams) -> Result<(), KafkaError> {
        todo!("send request acknowledge")
    }
}
