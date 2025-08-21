use anyhow::Result;
use rdkafka::{message::OwnedMessage, Message, Timestamp};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::error;

use crate::kafka::KafkaError;

/// MessageType defines the different types of messages that can be sent through Kafka.
/// It supports JSON serialization with custom names for each variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request message type - serializes to "REQUEST"
    #[serde(rename = "REQUEST")]
    Request,
    /// Response message type - serializes to "RESPONSE"
    #[serde(rename = "RESPONSE")]
    Response,
    /// General message type - serializes to "MESSAGE"
    #[serde(rename = "MESSAGE")]
    Message,
}

/// ResponseDestination holds the topic and URI for message responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDestination {
    pub topic: String,
    pub uri: String,
}

impl ResponseDestination {
    /// Determines if a response should be sent based on the presence of topic and URI.
    ///
    /// # Returns
    ///
    /// * `bool` - True if both topic and URI are non-empty, false otherwise.
    pub fn should_response(&self) -> bool {
        !self.topic.is_empty() && !self.uri.is_empty()
    }
}

/// ParsedMessage represents a parsed Kafka message with metadata and payload data.
/// It supports generic data types and includes fields for message routing and response handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMessage<T = serde_json::Value> {
    /// The type of message (Request, Response, or Message)
    pub message_type: MessageType,
    /// Optional source identifier
    pub source_id: String,
    /// Transaction identifier for tracking
    pub transaction_id: String,
    /// Unique message identifier
    pub message_id: String,
    /// URI for message routing
    pub uri: String,
    /// Optional response destination configuration
    pub response_destination: Option<ResponseDestination>,
    /// The data payload of the message
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessage<T = serde_json::Value> {
    pub topic: String,
    pub message: ParsedMessage<T>,
}

impl ParsedMessage {
    pub fn should_response(&self) -> bool {
        self.response_destination
            .as_ref()
            .map_or(false, |dest| dest.should_response())
    }

    pub fn parse_from_string(message: &str) -> Option<Self> {
        match serde_json::from_str::<ParsedMessage>(message) {
            Ok(parsed_message) => Some(parsed_message),
            Err(e) => {
                error!("Failed to parse JSON message: {}", e);
                None
            }
        }
    }

    pub fn get_response_destination(&self) -> Option<&ResponseDestination> {
        self.response_destination.as_ref()
    }

    pub fn get_data_as<U>(&self) -> Result<U>
    where
        U: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let inner = self.data.pointer("/data").ok_or_else(|| {
            anyhow::anyhow!(
                "missing nested data at /data path. Structure: {}",
                serde_json::to_string_pretty(&self.data).unwrap_or_else(|_| "invalid".to_string())
            )
        })?;

        let result = serde_json::from_value(inner.clone())?;
        tracing::info!("extracted data: {:?}", &result);
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status<T = serde_json::Value> {
    pub code: String,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<S = serde_json::Value, D = serde_json::Value> {
    pub status: Option<Status<S>>,
    pub data: Option<D>,
}

#[derive(Debug)]
pub enum HandlerResult {
    Response(serde_json::Value),
    Acknowledge,
}

pub type MessageHandler = Arc<
    dyn Fn(
            &ParsedMessage,
        ) -> Pin<Box<dyn Future<Output = Result<HandlerResult, KafkaError>> + Send>>
        + Send
        + Sync,
>;

pub trait MessageLatency {
    fn get_latency(&self) -> i64; // abstract method

    fn get_latency_formatted(&self) -> String {
        let latency = self.get_latency();
        if latency == 0 {
            "N/A".to_string()
        } else {
            format!("{}ms", latency)
        }
    }

    fn is_expired(&self, timeout_secs: i64) -> bool {
        let latency = self.get_latency();
        latency > 0 && latency > timeout_secs * 1000
    }
}

impl MessageLatency for OwnedMessage {
    fn get_latency(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        match self.timestamp() {
            Timestamp::CreateTime(ts) => now - ts,    // tính từ producer
            Timestamp::LogAppendTime(ts) => now - ts, // tính từ broker
            Timestamp::NotAvailable => 0,
        }
    }
}
