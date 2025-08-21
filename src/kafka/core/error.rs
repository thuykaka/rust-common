use crate::kafka::{Response, Status};

pub mod error_codes {
    pub const INTERNAL_SERVER_ERROR: &str = "INTERNAL_SERVER_ERROR";
    pub const URI_NOT_FOUND: &str = "URI_NOT_FOUND";
    pub const INVALID_PARAMETER: &str = "INVALID_PARAMETER";
    pub const FIELD_REQUIRED: &str = "FIELD_REQUIRED";
    pub const VALUE_INVALID: &str = "VALUE_INVALID";
    pub const TIMEOUT_ERROR: &str = "TIMEOUT_ERROR";
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const OBJECT_NOT_FOUND: &str = "OBJECT_NOT_FOUND";
    pub const SECOND_FACTOR_REQUIRED: &str = "SECOND_FACTOR_REQUIRED";
}

/// KafkaError defines the various errors that can occur within the Kafka module.
/// It provides structured error messages for different failure scenarios.
#[derive(thiserror::Error, Debug)]
pub enum KafkaError {
    /// Represents an internal server error with a detailed message.
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    /// Indicates that a requested URI was not found.
    #[error("Uri not found: {0}")]
    UriNotFound(String),

    /// Represents an error during message serialization.
    #[error("Serialization Error: {0}")]
    SerializationError(String),

    /// Represents a connection error with a detailed message.
    #[error("Connection Error: {0}")]
    ConnectionError(String),

    /// Indicates a timeout error with a detailed message.
    #[error("Timeout Error: {0}")]
    TimeoutError(String),

    /// Represents a configuration error with a detailed message.
    #[error("Configuration Error: {0}")]
    ConfigurationError(String),
}

impl KafkaError {
    /// Converts the KafkaError into a structured Response.
    ///
    /// # Returns
    ///
    /// * `Response` - A structured response containing the error code and message.
    pub fn to_response(&self) -> Response {
        match self {
            KafkaError::InternalServerError(_) => Response {
                status: Some(Status {
                    code: error_codes::INTERNAL_SERVER_ERROR.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
            KafkaError::UriNotFound(_) => Response {
                status: Some(Status {
                    code: error_codes::URI_NOT_FOUND.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
            KafkaError::SerializationError(_) => Response {
                status: Some(Status {
                    code: error_codes::VALUE_INVALID.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
            KafkaError::ConnectionError(_) => Response {
                status: Some(Status {
                    code: error_codes::TIMEOUT_ERROR.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
            KafkaError::TimeoutError(_) => Response {
                status: Some(Status {
                    code: error_codes::TIMEOUT_ERROR.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
            KafkaError::ConfigurationError(_) => Response {
                status: Some(Status {
                    code: error_codes::INVALID_PARAMETER.to_string(),
                    message: self.to_string(),
                    data: None,
                }),
                data: None,
            },
        }
    }

    /// Converts the KafkaError into a JSON value for response serialization.
    ///
    /// # Returns
    ///
    /// * `serde_json::Value` - A JSON representation of the error response.
    pub fn to_response_value(&self) -> serde_json::Value {
        serde_json::to_value(self.to_response()).unwrap_or_else(|_| {
            serde_json::json!({
                "status": {
                    "code": "INTERNAL_SERVER_ERROR",
                    "message": "Failed to serialize error response",
                    "data": null
                },
                "data": null
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_server_error() {
        let error = KafkaError::InternalServerError("Database connection failed".to_string());

        assert_eq!(
            error.to_string(),
            "Internal Server Error: Database connection failed"
        );

        let response = error.to_response();
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, error_codes::INTERNAL_SERVER_ERROR);
        assert_eq!(
            status.message,
            "Internal Server Error: Database connection failed"
        );
        assert!(status.data.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_uri_not_found_error() {
        let error = KafkaError::UriNotFound("/api/users/123".to_string());

        assert_eq!(error.to_string(), "Uri not found: /api/users/123");

        let response = error.to_response();
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, error_codes::URI_NOT_FOUND);
        assert_eq!(status.message, "Uri not found: /api/users/123");
        assert!(status.data.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_serialization_error() {
        let error = KafkaError::SerializationError("Invalid JSON format".to_string());

        assert_eq!(
            error.to_string(),
            "Serialization Error: Invalid JSON format"
        );

        let response = error.to_response();
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, error_codes::VALUE_INVALID);
        assert_eq!(status.message, "Serialization Error: Invalid JSON format");
        assert!(status.data.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_connection_error() {
        let error = KafkaError::ConnectionError("Kafka broker unreachable".to_string());

        assert_eq!(
            error.to_string(),
            "Connection Error: Kafka broker unreachable"
        );

        let response = error.to_response();
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, error_codes::TIMEOUT_ERROR);
        assert_eq!(status.message, "Connection Error: Kafka broker unreachable");
        assert!(status.data.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_timeout_error() {
        let error = KafkaError::TimeoutError("Request timed out after 30 seconds".to_string());

        assert_eq!(
            error.to_string(),
            "Timeout Error: Request timed out after 30 seconds"
        );

        let response = error.to_response();
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, error_codes::TIMEOUT_ERROR);
        assert_eq!(
            status.message,
            "Timeout Error: Request timed out after 30 seconds"
        );
        assert!(status.data.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_configuration_error() {
        let error = KafkaError::ConfigurationError("Missing bootstrap.servers".to_string());

        assert_eq!(
            error.to_string(),
            "Configuration Error: Missing bootstrap.servers"
        );

        let response = error.to_response();
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert_eq!(status.code, error_codes::INVALID_PARAMETER);
        assert_eq!(
            status.message,
            "Configuration Error: Missing bootstrap.servers"
        );
        assert!(status.data.is_none());
        assert!(response.data.is_none());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR");
        assert_eq!(error_codes::URI_NOT_FOUND, "URI_NOT_FOUND");
        assert_eq!(error_codes::INVALID_PARAMETER, "INVALID_PARAMETER");
        assert_eq!(error_codes::FIELD_REQUIRED, "FIELD_REQUIRED");
        assert_eq!(error_codes::VALUE_INVALID, "VALUE_INVALID");
        assert_eq!(error_codes::TIMEOUT_ERROR, "TIMEOUT_ERROR");
        assert_eq!(error_codes::UNAUTHORIZED, "UNAUTHORIZED");
        assert_eq!(error_codes::OBJECT_NOT_FOUND, "OBJECT_NOT_FOUND");
        assert_eq!(
            error_codes::SECOND_FACTOR_REQUIRED,
            "SECOND_FACTOR_REQUIRED"
        );
    }

    #[test]
    fn test_error_debug() {
        let error = KafkaError::InternalServerError("Test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InternalServerError"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_error_clone() {
        let error = KafkaError::TimeoutError("Test timeout".to_string());
        let cloned_error = format!("{}", error);
        assert_eq!(cloned_error, "Timeout Error: Test timeout");
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            KafkaError::InternalServerError("test".to_string()),
            KafkaError::UriNotFound("test".to_string()),
            KafkaError::SerializationError("test".to_string()),
            KafkaError::ConnectionError("test".to_string()),
            KafkaError::TimeoutError("test".to_string()),
            KafkaError::ConfigurationError("test".to_string()),
        ];

        for error in errors {
            let response = error.to_response();
            assert!(response.status.is_some());
            let status = response.status.unwrap();
            assert!(!status.code.is_empty());
            assert!(!status.message.is_empty());
            assert!(status.data.is_none());
            assert!(response.data.is_none());
        }
    }
}
