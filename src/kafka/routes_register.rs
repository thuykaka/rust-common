use std::{collections::HashMap, sync::Arc};

use std::sync::Mutex;
use tracing::{error, info};

use crate::kafka::{HandlerResult, KafkaError, MessageHandler, ParsedMessage};

/// A macro for creating routes in a more concise DSL-style syntax.
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::{routes, HandlerResult};
/// use serde_json::json;
///
/// let routes = routes![
///     "/api/v1/login" => |msg| async move {
///         Ok(HandlerResult::Response(json!({ "token": "123" })))
///     },
///     "/api/v1/register" => |msg| async move {
///         Ok(HandlerResult::Response(json!({ "token": "456" })))
///     }
/// ];
/// ```
#[macro_export]
macro_rules! routes {
    // Handle empty routes
    () => {
        $crate::kafka::RouteRegistry::new()
    };

    // Handle single route
    ($path:expr => $handler:expr) => {{
    let mut registry = $crate::kafka::RouteRegistry::new();
        registry.register($path, $handler);
        registry
    }};
    // Handle multiple routes
    ($path:expr => $handler:expr, $($rest_path:expr => $rest_handler:expr),+ $(,)?) => {{
        let mut registry = $crate::kafka::RouteRegistry::new();
        registry.register($path, $handler);
        $(
            registry.register($rest_path, $rest_handler);
        )+
        registry
    }};
}

#[derive(Clone)]
pub struct RouteRegistry {
    routes: Arc<Mutex<HashMap<String, MessageHandler>>>,
}

impl RouteRegistry {
    /// Creates a new empty route registry
    pub fn new() -> Self {
        Self {
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a handler for a specific URI pattern
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI pattern to register the handler for
    /// * `handler` - The async function to handle messages
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if registration was successful, or `Err` if it failed.
    /// This allows for method chaining.
    pub fn register<F, Fut>(&mut self, uri: &str, f: F) -> &mut Self
    where
        F: Fn(ParsedMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<HandlerResult, KafkaError>> + Send + 'static,
    {
        let handler = Arc::new(move |msg: &ParsedMessage| {
            let fut = f(msg.clone());
            Box::pin(fut)
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<HandlerResult, KafkaError>> + Send>,
                >
        });

        if let Ok(mut routes) = self.routes.lock() {
            routes.insert(uri.to_string(), handler);
            info!("registered handler for uri: {}", uri);
        } else {
            error!("Failed to acquire lock for routes");
        }

        self
    }

    /// Checks if a handler is registered for the given URI
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to check for registered handlers
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if a handler is registered for the URI, `Ok(false)` if not,
    /// or `Err` if the check cannot be performed.
    pub fn has_handler(&self, uri: &str) -> Result<bool, KafkaError> {
        let routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(routes.contains_key(uri))
    }

    /// Returns a list of all registered URI patterns
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<String>)` containing all registered URIs, or `Err` if the
    /// operation cannot be completed.
    pub fn get_registered_uris(&self) -> Result<Vec<String>, KafkaError> {
        let routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(routes.keys().cloned().collect())
    }

    /// Gets a handler for the specified URI
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to get the handler for
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(handler))` if a handler is found, `Ok(None)` if not,
    /// or `Err` if the operation cannot be completed.
    pub fn get_handler(&self, uri: &str) -> Result<Option<MessageHandler>, KafkaError> {
        let routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(routes.get(uri).cloned())
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::kafka::HandlerResult;

    #[tokio::test]
    async fn test_routes_macro_empty() {
        let registry = routes![];
        assert_eq!(registry.get_registered_uris().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_routes_macro_single_route() {
        let registry = routes![
            "/api/test" => |_msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({"status": "ok"})))
            }
        ];

        assert_eq!(registry.get_registered_uris().unwrap().len(), 1);
        assert!(registry.has_handler("/api/test").unwrap());
    }

    #[tokio::test]
    async fn test_routes_macro_multiple_routes() {
        let registry = routes![
            "/api/v1/login" => |_msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({"token": "123"})))
            },
            "/api/v1/register" => |_msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({"token": "456"})))
            },
            "/api/v1/profile" => |_msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({"user": "john"})))
            }
        ];

        let uris = registry.get_registered_uris().unwrap();
        assert_eq!(uris.len(), 3);
        assert!(registry.has_handler("/api/v1/login").unwrap());
        assert!(registry.has_handler("/api/v1/register").unwrap());
        assert!(registry.has_handler("/api/v1/profile").unwrap());
        assert!(!registry.has_handler("/api/v1/unknown").unwrap());
    }

    #[tokio::test]
    async fn test_routes_macro_with_trailing_comma() {
        let registry = routes![
            "/api/test1" => |_msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({"test": 1})))
            },
            "/api/test2" => |_msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({"test": 2})))
            },
        ];

        assert_eq!(registry.get_registered_uris().unwrap().len(), 2);
        assert!(registry.has_handler("/api/test1").unwrap());
        assert!(registry.has_handler("/api/test2").unwrap());
    }

    #[tokio::test]
    async fn test_routes_macro_handler_execution() {
        let registry = routes![
            "/api/echo" => |msg| async move {
                Ok(HandlerResult::Response(serde_json::json!({
                    "echo": msg.uri,
                    "message": "received"
                })))
            }
        ];

        let handler = registry.get_handler("/api/echo").unwrap().unwrap();
        let test_msg = crate::kafka::ParsedMessage {
            message_type: crate::kafka::core::MessageType::Request,
            source_id: Some("test-service".to_string()),
            transaction_id: "tx-123".to_string(),
            message_id: "msg-456".to_string(),
            uri: "/api/echo".to_string(),
            response_destination: None,
            data: serde_json::Value::Null,
        };

        let result = handler(&test_msg).await.unwrap();
        match result {
            HandlerResult::Response(response) => {
                assert_eq!(response["echo"], "/api/echo");
                assert_eq!(response["message"], "received");
            }
            _ => panic!("Expected Response variant"),
        }
    }
}
