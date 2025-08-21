use std::{collections::HashMap, sync::Arc};

use std::sync::Mutex;
use tracing::{error, info};

use crate::kafka::{HandlerResult, KafkaError, MessageHandler, ParsedMessage};

/// The `routes` macro provides a convenient way to create a `RouteRegistry` with registered routes.
///
/// # Examples
///
/// ```rust
/// use rust_common::kafka::routes;
///
/// let registry = routes!(
///     "/api/users" => user_handler,
///     "/api/orders" => order_handler
/// );
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

/// `RouteRegistry` manages the registration and retrieval of message handlers for specific URIs.
#[derive(Clone)]
pub struct RouteRegistry {
    routes: Arc<Mutex<HashMap<String, MessageHandler>>>,
}

impl RouteRegistry {
    /// Creates a new, empty `RouteRegistry`.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of `RouteRegistry`.
    pub fn new() -> Self {
        Self {
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a message handler for a specific URI.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI for which the handler is registered.
    /// * `f` - The handler function to register.
    ///
    /// # Returns
    ///
    /// * `&mut Self` - The updated `RouteRegistry` instance.
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

    /// Checks if a handler is registered for a specific URI.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to check.
    ///
    /// # Returns
    ///
    /// * `Result<bool, KafkaError>` - Returns true if a handler is registered, false otherwise.
    pub fn has_handler(&self, uri: &str) -> Result<bool, KafkaError> {
        let routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(routes.contains_key(uri))
    }

    /// Retrieves all registered URIs.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, KafkaError>` - A vector of registered URIs.
    pub fn get_registered_uris(&self) -> Result<Vec<String>, KafkaError> {
        let routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(routes.keys().cloned().collect())
    }

    /// Retrieves the handler for a specific URI, if it exists.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI for which to retrieve the handler.
    ///
    /// # Returns
    ///
    /// * `Result<Option<MessageHandler>, KafkaError>` - The handler if it exists, or None.
    pub fn get_handler(&self, uri: &str) -> Result<Option<MessageHandler>, KafkaError> {
        let routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(routes.get(uri).cloned())
    }
}

impl Default for RouteRegistry {
    /// Creates a default instance of `RouteRegistry`.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of `RouteRegistry`.
    fn default() -> Self {
        Self::new()
    }
}
