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

/// Message types for Kafka messages
///
/// This enum defines the different types of messages that can be sent through Kafka,
/// with proper JSON serialization support.
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::core::MessageType;
///
/// let message_type = MessageType::Request;
/// // Serializes to "REQUEST" in JSON
/// ```
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

/// Parsed message structure for Kafka messages
///
/// This struct represents a parsed Kafka message with metadata and payload data.
/// It supports generic data types and includes fields for message routing and response handling.
///
/// # Generic Parameters
///
/// * `T` - The data type for the message payload (defaults to `serde_json::Value`)
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::core::*;
/// use serde_json::json;
///
/// let message = ParsedMessage {
///     message_type: MessageType::Request,
///     source_id: Some("service-a".to_string()),
///     transaction_id: "tx-123".to_string(),
///     message_id: "msg-456".to_string(),
///     uri: "/api/users".to_string(),
///     response_destination: Some(ResponseDestination {
///         topic: "responses".to_string(),
///         uri: "/api/users/response".to_string(),
///     }),
///     data: json!({
///         "user_id": 123,
///         "action": "create"
///     }),
/// };
///
/// // Parse from JSON string
/// let json_str = serde_json::to_string(&message)?;
/// let parsed = ParsedMessage::parse_from_string(&json_str);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMessage<T = serde_json::Value> {
    /// The type of message (Request, Response, or Message)
    pub message_type: MessageType,
    /// Optional source identifier
    pub source_id: Option<String>,
    /// Transaction identifier for tracking
    pub transaction_id: String,
    /// Unique message identifier
    pub message_id: String,
    /// URI for message routing
    pub uri: String,
    /// Optional response destination configuration
    pub response_destination: Option<ResponseDestination>,
    /// The actual message data/payload
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

    fn is_expired(&self, expire_time: i64) -> bool {
        let latency = self.get_latency();
        latency > 0 && latency > expire_time
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_type_serialization() {
        let request = MessageType::Request;
        let response = MessageType::Response;
        let message = MessageType::Message;

        let request_json = serde_json::to_string(&request).unwrap();
        let response_json = serde_json::to_string(&response).unwrap();
        let message_json = serde_json::to_string(&message).unwrap();

        assert_eq!(request_json, "\"REQUEST\"");
        assert_eq!(response_json, "\"RESPONSE\"");
        assert_eq!(message_json, "\"MESSAGE\"");
    }

    #[test]
    fn test_message_type_deserialization() {
        let request: MessageType = serde_json::from_str("\"REQUEST\"").unwrap();
        let response: MessageType = serde_json::from_str("\"RESPONSE\"").unwrap();
        let message: MessageType = serde_json::from_str("\"MESSAGE\"").unwrap();

        assert!(matches!(request, MessageType::Request));
        assert!(matches!(response, MessageType::Response));
        assert!(matches!(message, MessageType::Message));
    }

    #[test]
    fn test_response_destination() {
        let dest = ResponseDestination {
            topic: "responses".to_string(),
            uri: "/api/response".to_string(),
        };

        assert!(dest.should_response());
        assert_eq!(dest.topic, "responses");
        assert_eq!(dest.uri, "/api/response");
    }

    #[test]
    fn test_response_destination_empty() {
        let dest = ResponseDestination {
            topic: "".to_string(),
            uri: "/api/response".to_string(),
        };

        assert!(!dest.should_response());

        let dest2 = ResponseDestination {
            topic: "responses".to_string(),
            uri: "".to_string(),
        };

        assert!(!dest2.should_response());
    }

    #[test]
    fn test_parsed_message_creation() {
        let message = ParsedMessage {
            message_type: MessageType::Request,
            source_id: Some("service-a".to_string()),
            transaction_id: "tx-123".to_string(),
            message_id: "msg-456".to_string(),
            uri: "/api/users".to_string(),
            response_destination: Some(ResponseDestination {
                topic: "responses".to_string(),
                uri: "/api/users/response".to_string(),
            }),
            data: json!({
                "user_id": 123,
                "action": "create"
            }),
        };

        assert!(matches!(message.message_type, MessageType::Request));
        assert_eq!(message.source_id, Some("service-a".to_string()));
        assert_eq!(message.transaction_id, "tx-123");
        assert_eq!(message.message_id, "msg-456");
        assert_eq!(message.uri, "/api/users");
        assert!(message.should_response());
    }

    #[test]
    fn test_parsed_message_should_response() {
        let message_with_response = ParsedMessage {
            message_type: MessageType::Request,
            source_id: None,
            transaction_id: "tx-123".to_string(),
            message_id: "msg-456".to_string(),
            uri: "/api/users".to_string(),
            response_destination: Some(ResponseDestination {
                topic: "responses".to_string(),
                uri: "/api/users/response".to_string(),
            }),
            data: json!({}),
        };

        let message_without_response = ParsedMessage {
            message_type: MessageType::Request,
            source_id: None,
            transaction_id: "tx-123".to_string(),
            message_id: "msg-456".to_string(),
            uri: "/api/users".to_string(),
            response_destination: None,
            data: json!({}),
        };

        assert!(message_with_response.should_response());
        assert!(!message_without_response.should_response());
    }

    #[test]
    fn test_parse_from_string_valid() {
        let json_str = r#"{
            "messageType": "REQUEST",
            "sourceId": "service-a",
            "transactionId": "tx-123",
            "messageId": "msg-456",
            "uri": "/api/users",
            "responseDestination": {
                "topic": "responses",
                "uri": "/api/users/response"
            },
            "data": {"user_id": 123}
        }"#;

        let parsed = ParsedMessage::parse_from_string(json_str);
        assert!(parsed.is_some());

        let message = parsed.unwrap();
        assert!(matches!(message.message_type, MessageType::Request));
        assert_eq!(message.source_id, Some("service-a".to_string()));
        assert_eq!(message.transaction_id, "tx-123");
        assert_eq!(message.message_id, "msg-456");
        assert_eq!(message.uri, "/api/users");
        assert!(message.should_response());
    }

    #[test]
    fn test_parse_from_string_invalid() {
        let invalid_json = "{ invalid json }";
        let parsed = ParsedMessage::parse_from_string(invalid_json);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_from_string_missing_fields() {
        let incomplete_json = r#"{
            "messageType": "REQUEST",
            "transactionId": "tx-123"
        }"#;

        let parsed = ParsedMessage::parse_from_string(incomplete_json);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_send_message() {
        let parsed_message = ParsedMessage {
            message_type: MessageType::Request,
            source_id: None,
            transaction_id: "tx-123".to_string(),
            message_id: "msg-456".to_string(),
            uri: "/api/users".to_string(),
            response_destination: None,
            data: json!({}),
        };

        let send_message = SendMessage {
            topic: "users".to_string(),
            message: parsed_message,
        };

        assert_eq!(send_message.topic, "users");
        assert!(matches!(
            send_message.message.message_type,
            MessageType::Request
        ));
    }

    #[test]
    fn test_status_and_response() {
        let status = Status {
            code: "SUCCESS".to_string(),
            message: "Operation completed".to_string(),
            data: Some(json!({"result": "ok"})),
        };

        let response = Response {
            status: Some(status),
            data: Some(json!({"data": "value"})),
        };

        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, "SUCCESS");
        assert_eq!(status.message, "Operation completed");
        assert!(status.data.is_some());
        assert!(response.data.is_some());
    }

    #[test]
    fn test_handler_result() {
        let response_result = HandlerResult::Response(json!({"data": "value"}));
        let ack_result = HandlerResult::Acknowledge;

        assert!(matches!(response_result, HandlerResult::Response(_)));
        assert!(matches!(ack_result, HandlerResult::Acknowledge));
    }

    #[test]
    fn test_message_latency_trait() {
        // Create a mock OwnedMessage for testing
        // Note: This is a simplified test since we can't easily create OwnedMessage instances
        // In a real scenario, you'd use a mock or test with actual Kafka messages

        // Test the default implementation
        struct MockMessage;

        impl MessageLatency for MockMessage {
            fn get_latency(&self) -> i64 {
                100 // Mock latency
            }
        }

        let mock_message = MockMessage;
        assert_eq!(mock_message.get_latency(), 100);
        assert_eq!(mock_message.get_latency_formatted(), "100ms");
    }

    #[test]
    fn test_message_latency_zero() {
        struct MockMessage;

        impl MessageLatency for MockMessage {
            fn get_latency(&self) -> i64 {
                0
            }
        }

        let mock_message = MockMessage;
        assert_eq!(mock_message.get_latency(), 0);
        assert_eq!(mock_message.get_latency_formatted(), "N/A");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original_message = ParsedMessage {
            message_type: MessageType::Response,
            source_id: Some("service-b".to_string()),
            transaction_id: "tx-789".to_string(),
            message_id: "msg-999".to_string(),
            uri: "/api/orders".to_string(),
            response_destination: None,
            data: json!({
                "order_id": "order-123",
                "status": "completed"
            }),
        };

        // Serialize
        let json_str = serde_json::to_string(&original_message).unwrap();

        // Deserialize
        let parsed = ParsedMessage::parse_from_string(&json_str);
        assert!(parsed.is_some());

        let deserialized_message = parsed.unwrap();
        assert!(matches!(
            deserialized_message.message_type,
            MessageType::Response
        ));
        assert_eq!(
            deserialized_message.source_id,
            Some("service-b".to_string())
        );
        assert_eq!(deserialized_message.transaction_id, "tx-789");
        assert_eq!(deserialized_message.message_id, "msg-999");
        assert_eq!(deserialized_message.uri, "/api/orders");
    }
}
