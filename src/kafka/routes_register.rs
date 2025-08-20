use std::{collections::HashMap, sync::Arc};

use std::sync::Mutex;
use tracing::info;

use crate::kafka::{HandlerResult, KafkaError, MessageHandler, ParsedMessage};

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
    pub fn register<F, Fut>(&self, uri: &str, f: F) -> Result<Self, KafkaError>
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

        let mut routes = self
            .routes
            .lock()
            .map_err(|_| KafkaError::InternalServerError("Failed to acquire lock".to_string()))?;

        routes.insert(uri.to_string(), handler);
        info!("registered handler for uri: {}", uri);

        Ok(Self {
            routes: Arc::clone(&self.routes),
        })
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
