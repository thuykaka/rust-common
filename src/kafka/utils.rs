pub mod utils {
    use rdkafka::{message::OwnedMessage, Message};
    use tracing::warn;

    use crate::kafka::{MessageType, ParsedMessage, ResponseDestination, SendMessage};

    /// Extracts the payload from an OwnedMessage as a String.
    ///
    /// # Arguments
    ///
    /// * `message` - The Kafka message from which to extract the payload.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The payload as a String if it exists and is valid, otherwise None.
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

    /// Creates a SendMessage with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `source_id` - The source identifier for the message.
    /// * `message_id` - The unique message identifier.
    /// * `transaction_id` - The transaction identifier for tracking.
    /// * `topic` - The topic to which the message will be sent.
    /// * `uri` - The URI for message routing.
    /// * `data` - The data payload of the message.
    /// * `message_type` - The type of message (optional).
    /// * `response_destination` - The response destination configuration (optional).
    ///
    /// # Returns
    ///
    /// * `SendMessage` - A structured message ready to be sent to Kafka.
    pub fn create_message(
        source_id: String,
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
