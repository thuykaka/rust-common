pub mod utils {
    use rdkafka::{message::OwnedMessage, Message};
    use tracing::warn;

    use crate::kafka::{MessageType, ParsedMessage, ResponseDestination, SendMessage};

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
