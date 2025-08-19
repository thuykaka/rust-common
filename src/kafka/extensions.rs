use rdkafka::message::OwnedMessage;
use rdkafka::{Message, Timestamp};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;

pub trait MessageLatency {
    fn get_latency(&self) -> i64;

    fn get_latency_formatted(&self) -> String {
        let latency = self.get_latency();

        if latency == 0 {
            "N/A".to_string()
        } else {
            format!("{}ms", latency)
        }
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

pub mod utils {
    use tracing::warn;

    use super::*;

    pub fn extract_payload(message: &OwnedMessage) -> Option<String> {
        match message.payload_view::<str>() {
            Some(Ok(payload)) => Some(payload.to_string()),
            Some(Err(_)) => {
                warn!("invalid payload from topic {}", message.topic());
                None
            }
            None => None,
        }
    }

    pub fn create_message(
        source_id: Option<String>,
        message_id: String,
        transaction_id: String,
        topic: String,
        uri: String,
        data: serde_json::Value,
        message_type: Option<MessageType>,
        response_destination: Option<ResponseDestination>,
    ) -> SendMessage {
        SendMessage {
            topic,
            message: ParsedMessage {
                message_type: message_type.unwrap_or(MessageType::Message),
                source_id,
                message_id,
                transaction_id,
                uri,
                response_destination,
                data,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "REQUEST")]
    Request,
    #[serde(rename = "RESPONSE")]
    Response,
    #[serde(rename = "MESSAGE")]
    Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMessage<T = serde_json::Value> {
    pub message_type: MessageType,
    pub source_id: Option<String>,
    pub transaction_id: String,
    pub message_id: String,
    pub uri: String,
    pub response_destination: Option<ResponseDestination>,
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

    pub fn parse_from_string(message: &str) -> anyhow::Result<Self> {
        let parsed_message: ParsedMessage = serde_json::from_str(message).map_err(|e| {
            error!("Failed to parse JSON message: {}", e);
            anyhow::anyhow!("Failed to parse JSON message: {}", e)
        })?;

        Ok(parsed_message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDestination {
    pub topic: String,
    pub uri: String,
}

impl ResponseDestination {
    pub fn should_response(&self) -> bool {
        !self.topic.is_empty() && !self.uri.is_empty()
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
