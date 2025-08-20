// use std::{collections::HashMap, sync::Arc, time::Instant};

// use crate::kafka::{
//     utils::{create_message, extract_payload},
//     CustomContext, KafkaConfig, KafkaError, MessageType, ParsedMessage,
// };
// use anyhow::{Context, Result};
// use futures::StreamExt;
// use rdkafka::{
//     config::RDKafkaLogLevel,
//     consumer::Consumer,
//     message::OwnedMessage,
//     producer::{FutureProducer, FutureRecord},
//     ClientConfig, Message,
// };
// use tokio::sync::{oneshot, RwLock};
// use tracing::{info, warn};

// use crate::kafka::{extensions::MessageLatency, LoggingConsumer};

// struct PendingRequest {
//     sender: oneshot::Sender<ParsedMessage>,
//     created_at: std::time::Instant,
//     timeout_task: tokio::task::JoinHandle<()>,
// }

// pub struct RequestSender {
//     config: KafkaConfig,
//     consumer: LoggingConsumer,
//     producer: Arc<FutureProducer>,
//     concurrency_limit: usize,
//     pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
//     response_topic: String,
//     expire_time: i64,
//     ping_interval_secs: u64,
// }

// impl RequestSender {
//     pub fn new(config: KafkaConfig) -> anyhow::Result<Self> {
//         Self::with_concurrency_limit(config, 100, 60000)
//     }

//     pub fn with_concurrency_limit(
//         config: KafkaConfig,
//         concurrency_limit: usize,
//         expire_time: i64,
//     ) -> Result<Self> {
//         let response_topic = format!("{}.response.{}", &config.cluster_id, uuid::Uuid::new_v4());

//         let context = CustomContext;

//         let mut consumer_config = ClientConfig::new();

//         consumer_config
//             .set(
//                 "group.id",
//                 format!("{}-{}", &config.cluster_id, uuid::Uuid::new_v4()),
//             )
//             .set("bootstrap.servers", &config.bootstrap_servers)
//             .set("allow.auto.create.topics", "true")
//             .set("enable.partition.eof", "false")
//             .set("enable.auto.commit", "true")
//             .set("auto.offset.reset", "earliest")
//             .set("session.timeout.ms", "10000")
//             .set("heartbeat.interval.ms", "500")
//             .set_log_level(RDKafkaLogLevel::Debug);

//         let consumer: LoggingConsumer = consumer_config
//             .create_with_context(context)
//             .context("Consumer creation failed")?;

//         let topics: Vec<&str> = vec![&response_topic];

//         consumer
//             .subscribe(&topics)
//             .context("Can't subscribe to specified topics")?;

//         info!("subscribed to response topic: {}", response_topic);

//         let mut producer_config = ClientConfig::new();

//         producer_config
//             .set("bootstrap.servers", &config.bootstrap_servers)
//             .set("message.timeout.ms", "5000")
//             .set("allow.auto.create.topics", "true")
//             .set("acks", "0")
//             .set_log_level(RDKafkaLogLevel::Debug);

//         let producer: FutureProducer = producer_config
//             .create()
//             .context("Producer creation failed")?;

//         Ok(Self {
//             config,
//             consumer,
//             producer: Arc::new(producer),
//             concurrency_limit,
//             pending_requests: Arc::new(RwLock::new(HashMap::new())),
//             response_topic,
//             expire_time,
//             ping_interval_secs: 30, // Default 30 seconds
//         })
//     }

//     pub fn get_config(&self) -> &KafkaConfig {
//         &self.config
//     }

//     pub fn get_concurrency_limit(&self) -> usize {
//         self.concurrency_limit
//     }

//     pub fn set_concurrency_limit(&mut self, limit: usize) {
//         self.concurrency_limit = limit;
//     }

//     pub async fn start_processing(&self) -> anyhow::Result<()> {
//         info!("starting message processing...");

//         self.consumer
//             .stream()
//             .filter_map(|msg| async { msg.ok() })
//             .map(|m| {
//                 let pending_requests = Arc::clone(&self.pending_requests);
//                 let expire_time = self.expire_time;
//                 async move {
//                     Self::process_message(m.detach(), pending_requests, expire_time).await
//                 }
//             })
//             .buffer_unordered(self.concurrency_limit)
//             .for_each(|res| async {
//                 if let Err(e) = res {
//                     warn!("Error processing message: {}", e);
//                 }
//             })
//             .await;

//         Ok(())
//     }

//     pub async fn ping(&self) -> Result<()> {
//         Self::send_ping(&self.producer, &self.response_topic).await
//     }

//     async fn send_ping(producer: &Arc<FutureProducer>, response_topic: &str) -> Result<()> {
//         info!("ping");

//         let send_message = create_message(
//             None,
//             "".to_string(),
//             uuid::Uuid::new_v4().to_string(),
//             response_topic.to_string(),
//             "ping".to_string(),
//             serde_json::Value::Null,
//             None,
//             None,
//         );

//         let payload = serde_json::to_string(&send_message.message).map_err(|e| {
//             KafkaError::InternalServerError(format!("Failed to serialize response message: {}", e))
//         })?;

//         let record = FutureRecord::<String, String>::to(&send_message.topic).payload(&payload);

//         let _ = producer
//             .as_ref()
//             .send(record, std::time::Duration::from_secs(0))
//             .await
//             .map_err(|(e, _)| {
//                 tracing::error!(
//                     "send request {}, message: {} to topic {} failed: {}",
//                     send_message.message.transaction_id,
//                     payload,
//                     send_message.topic,
//                     e
//                 );
//                 KafkaError::InternalServerError(format!("Failed to send ping message: {}", e))
//             })?;

//         info!(
//             "send request {}, message: {} to topic {} success",
//             send_message.message.transaction_id, payload, send_message.topic
//         );

//         Ok(())
//     }

//     async fn process_message(
//         message: OwnedMessage,
//         pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
//         expire_time: i64,
//     ) -> anyhow::Result<()> {
//         let message_value =
//             extract_payload(&message).ok_or_else(|| anyhow::anyhow!("message has no payload"))?;

//         let latency = message.get_latency();
//         info!(
//             "received message: '{}' from topic {}, latency: {}ms",
//             message_value,
//             message.topic(),
//             latency,
//         );

//         if latency > 0 && latency > expire_time {
//             warn!(
//                 "ignore this request because it is expired {} _ {}",
//                 expire_time, message_value
//             );
//             return Ok(());
//         }

//         let parsed_message = ParsedMessage::parse_from_string(&message_value)
//             .context("failed to parse message from Kafka payload")?;

//         if parsed_message.uri == "ping" {
//             return Self::handle_ping();
//         }

//         let mut guard = pending_requests.write().await;

//         if let Some(request) = guard.remove(&parsed_message.transaction_id) {
//             let duration = Instant::now()
//                 .duration_since(request.created_at)
//                 .as_millis();

//             info!(
//                 "request {} took {}ms",
//                 parsed_message.transaction_id, duration
//             );

//             let _ = request.sender.send(parsed_message);
//             request.timeout_task.abort();
//         } else {
//             warn!(
//                 "ignore this request because it is not found {} (maybe timeout)",
//                 parsed_message.transaction_id
//             );
//             return Ok(());
//         }

//         Ok(())
//     }

//     fn handle_ping() -> Result<()> {
//         info!("pong");
//         Ok(())
//     }
// }
