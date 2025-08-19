use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use futures::StreamExt;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance, StreamConsumer},
    message::OwnedMessage,
    ClientConfig, ClientContext, Message, Timestamp,
};
use rust_common::logger::{self};
use tracing::{info, warn};

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

// Emulates an expensive, synchronous computation.
fn expensive_computation(msg: OwnedMessage) -> String {
    info!("Starting expensive computation on message {}", msg.offset());

    let message = match msg.payload_view::<str>() {
        Some(Ok(payload)) => format!("Payload len for {} is {}", payload, payload.len()),
        Some(Err(_)) => "Message payload is not a string".to_owned(),
        None => "No payload".to_owned(),
    };

    info!("{}", message);

    message
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_with_default().context("Failed to initialize logger")?;

    info!("Starting Kafka consumer demo");

    let context = CustomContext;

    let mut config = ClientConfig::new();

    // config: https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md
    config
        .set("group.id", "rust-kafka-consumer")
        .set("bootstrap.servers", "localhost:9092")
        .set("allow.auto.create.topics", "true")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "10000")
        .set("heartbeat.interval.ms", "500")
        .set_log_level(RDKafkaLogLevel::Debug);

    let consumer: LoggingConsumer = config
        .create_with_context(context)
        .context("Consumer creation failed")?;

    consumer
        .subscribe(&["topic_name"])
        .context("Can't subscribe to specified topics")?;

    // Use a more robust error handling approach
    let stream_processor = consumer.stream().for_each(|result| async {
        match result {
            Err(e) => {
                warn!("Error receiving message: {}", e);
            }
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    Some(Ok(payload)) => payload,
                    Some(Err(_)) => "<invalid payload>",
                    None => "<no payload>",
                };

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;

                let latency = match m.timestamp() {
                    Timestamp::CreateTime(ts) => now - ts,    // tính từ producer
                    Timestamp::LogAppendTime(ts) => now - ts, // tính từ broker
                    Timestamp::NotAvailable => 0,
                };

                info!(
                    "📩 Received: '{}' from topic {} [{}] offset {}, latency: {}ms",
                    payload,
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    latency
                );

                // Optional: Process message asynchronously for expensive operations
                let owned_message = m.detach();
                tokio::spawn(async move {
                    let computation_result =
                        tokio::task::spawn_blocking(|| expensive_computation(owned_message))
                            .await
                            .unwrap_or_else(|_| "Failed to process message".to_string());

                    info!("Processed: {}", computation_result);
                });
            }
        }
    });

    // Run the stream processor
    stream_processor.await;

    // Graceful shutdown - StreamConsumer will be dropped automatically
    info!("Shutting down Kafka consumer");

    Ok(())
}
