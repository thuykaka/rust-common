use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, Producer},
    ClientConfig, Message as KafkaMessage,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::kafka::{
    config::KafkaConfig,
    message::{
        KafkaMessage as CustomKafkaMessage, Message, MessageType, Response, ResponseDestination,
    },
};

pub type MessageHandler =
    Arc<dyn Fn(Message) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;
pub type AsyncMessageHandler = Box<
    dyn Fn(Message) -> tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + Sync,
>;

pub struct ConsumerHandler {
    consumer: StreamConsumer,
    producer: FutureProducer,
    workers: usize,
    routes: Arc<Mutex<HashMap<String, MessageHandler>>>,
    topics: Vec<String>,
    kafka_config: KafkaConfig,
}

impl ConsumerHandler {
    pub fn new(
        kafka_config: KafkaConfig,
        consumer_config: Option<ConsumerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let workers = consumer_config.as_ref().map(|c| c.workers).unwrap_or(16);
        let topics = consumer_config
            .and_then(|c| c.topics)
            .unwrap_or_else(|| vec![kafka_config.cluster_id.clone()]);

        // Create consumer config
        let mut consumer_config = ClientConfig::new();
        consumer_config.set("bootstrap.servers", &kafka_config.bootstrap_servers);
        consumer_config.set("group.id", &kafka_config.cluster_id);
        consumer_config.set(
            "client.id",
            &format!("consumer-{}-{}", kafka_config.cluster_id, Uuid::new_v4()),
        );
        consumer_config.set("enable.auto.commit", "true");
        consumer_config.set("auto.offset.reset", "earliest");
        consumer_config.set("enable.partition.eof", "false");
        consumer_config.set("session.timeout.ms", "10000");
        consumer_config.set("heartbeat.interval.ms", "500");

        // Create producer config
        let mut producer_config = ClientConfig::new();
        producer_config.set("bootstrap.servers", &kafka_config.bootstrap_servers);
        producer_config.set(
            "client.id",
            &format!("producer-{}-{}", kafka_config.cluster_id, Uuid::new_v4()),
        );
        producer_config.set("acks", "0");

        let consumer: StreamConsumer = consumer_config.create()?;
        let producer: FutureProducer = producer_config.create()?;

        Ok(Self {
            consumer,
            producer,
            workers,
            routes: Arc::new(Mutex::new(HashMap::new())),
            topics,
            kafka_config,
        })
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();

        // Subscribe to topics
        self.consumer
            .subscribe(&self.topics.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;

        let duration = start_time.elapsed();
        info!(
            "kafka consumer handler for topics {:?} started with workers {} took {:?}",
            self.topics, self.workers, duration
        );

        // Start message processing
        self.start_message_processing().await?;

        Ok(())
    }

    async fn start_message_processing(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let routes = Arc::clone(&self.routes);
        let producer = self.producer.clone();
        let kafka_config = self.kafka_config.clone();

        tokio::spawn(async move {
            loop {
                match tokio::time::timeout(
                    Duration::from_secs(1),
                    Self::receive_message(&routes, &producer, &kafka_config),
                )
                .await
                {
                    Ok(result) => {
                        if let Err(e) = result {
                            error!("Error processing message: {}", e);
                        }
                    }
                    Err(_) => {
                        // Timeout, continue
                    }
                }
            }
        });

        Ok(())
    }

    async fn receive_message(
        routes: &Arc<Mutex<HashMap<String, MessageHandler>>>,
        producer: &FutureProducer,
        kafka_config: &KafkaConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation
        // In a real implementation, you would use the consumer to receive messages
        // For now, we'll just return Ok to avoid blocking
        Ok(())
    }

    pub async fn register<F>(
        &self,
        uri: String,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(Message) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        info!("register handler for uri {}", uri);
        let mut routes = self.routes.lock().await;
        routes.insert(uri, Arc::new(handler));
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        // Check if consumer is still active
        // This would need to be implemented based on the actual Kafka client state
        true
    }

    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Close consumer
        Ok(())
    }

    fn should_response(message: &Message) -> bool {
        message.response_destination.is_some()
    }

    async fn send_response(
        producer: &FutureProducer,
        message_id: Option<String>,
        transaction_id: String,
        topic: String,
        uri: String,
        data: Response,
        kafka_config: &KafkaConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = CustomKafkaMessage::new(
            MessageType::Response,
            Some(kafka_config.cluster_id.clone()),
            transaction_id,
            message_id,
            Some(uri),
            topic,
            None,
            Some(serde_json::to_value(data)?),
        );

        let msg_content = serde_json::to_string(&message.message)?;
        info!("send message {} to topic {}", msg_content, message.topic);

        // Send message using producer
        let delivery_status = producer
            .send(
                rdkafka::producer::FutureRecord::to(&message.topic)
                    .payload(msg_content.as_bytes())
                    .key(""),
                Duration::from_secs(5),
            )
            .await;

        match delivery_status {
            Ok(_) => {
                info!("send response to topic {} success", message.topic);
            }
            Err((e, _)) => {
                error!("send response to topic {} failed: {}", message.topic, e);
            }
        }

        Ok(())
    }

    async fn message_handler(
        message: rdkafka::message::BorrowedMessage<'_>,
        routes: &Arc<Mutex<HashMap<String, MessageHandler>>>,
        producer: &FutureProducer,
        kafka_config: &KafkaConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message_value = match message.payload() {
            Some(payload) => String::from_utf8_lossy(payload).to_string(),
            None => {
                warn!("ignoring message with no value");
                return Ok(());
            }
        };

        let start_time = Instant::now();
        info!(
            "received message {}, latency: {:?}",
            message_value,
            start_time.elapsed()
        );

        let parsed_message: Message = serde_json::from_str(&message_value)?;
        let should_response = Self::should_response(&parsed_message);

        let uri = parsed_message.uri.clone().unwrap_or_default();
        let transaction_id = parsed_message.transaction_id.clone();
        let message_id = parsed_message.message_id.clone();
        let response_destination = parsed_message.response_destination.clone();

        let handler = {
            let routes_guard = routes.lock().await;
            routes_guard.get(&uri).cloned()
        };

        match handler {
            None => {
                if should_response {
                    let duration = start_time.elapsed();
                    info!(
                        "1. handling request {} - {} took {:?}",
                        uri, transaction_id, duration
                    );

                    // Send URI not found error
                    let error_response = Response {
                        status: Some(crate::kafka::message::Status {
                            code: "404".to_string(),
                            message: "URI not found".to_string(),
                            data: None,
                        }),
                        data: None,
                    };

                    if let Some(response_dest) = &response_destination {
                        Self::send_response(
                            producer,
                            message_id.clone(),
                            transaction_id.clone(),
                            response_dest.topic.clone(),
                            response_dest.uri.clone(),
                            error_response,
                            kafka_config,
                        )
                        .await?;
                    }
                }
            }
            Some(handler) => {
                let message_clone = parsed_message.clone();

                match handler(message_clone) {
                    Ok(true) => {
                        let duration = start_time.elapsed();
                        info!(
                            "2. forward request {} - {} took {:?}",
                            parsed_message.uri.as_deref().unwrap_or("unknown"),
                            parsed_message.transaction_id,
                            duration
                        );
                    }
                    Ok(false) => {
                        // Handle async execution
                        if should_response {
                            if let Some(response_dest) = &parsed_message.response_destination {
                                let success_response = Response {
                                    status: Some(crate::kafka::message::Status {
                                        code: "200".to_string(),
                                        message: "Success".to_string(),
                                        data: None,
                                    }),
                                    data: None,
                                };

                                Self::send_response(
                                    producer,
                                    parsed_message.message_id.clone(),
                                    parsed_message.transaction_id.clone(),
                                    response_dest.topic.clone(),
                                    response_dest.uri.clone(),
                                    success_response,
                                    kafka_config,
                                )
                                .await?;
                            }
                        }

                        let duration = start_time.elapsed();
                        info!(
                            "4. handle request {} - {} took {:?}",
                            parsed_message.uri.as_deref().unwrap_or("unknown"),
                            parsed_message.transaction_id,
                            duration
                        );
                    }
                    Err(e) => {
                        error!(
                            "Error while handling request {} - {}: {}",
                            parsed_message.uri.as_deref().unwrap_or("unknown"),
                            parsed_message.transaction_id,
                            e
                        );

                        if should_response {
                            if let Some(response_dest) = &parsed_message.response_destination {
                                let error_response = Response {
                                    status: Some(crate::kafka::message::Status {
                                        code: "500".to_string(),
                                        message: format!("Error: {}", e),
                                        data: None,
                                    }),
                                    data: None,
                                };

                                Self::send_response(
                                    producer,
                                    parsed_message.message_id.clone(),
                                    parsed_message.transaction_id.clone(),
                                    response_dest.topic.clone(),
                                    response_dest.uri.clone(),
                                    error_response,
                                    kafka_config,
                                )
                                .await?;
                            }
                        }

                        let duration = start_time.elapsed();
                        info!(
                            "3. handle request {} - {} took {:?}",
                            parsed_message.uri.as_deref().unwrap_or("unknown"),
                            parsed_message.transaction_id,
                            duration
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    pub workers: usize,
    pub topics: Option<Vec<String>>,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            workers: 16,
            topics: None,
        }
    }
}
