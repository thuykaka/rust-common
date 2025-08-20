//! # Kafka Core Module
//!
//! This module provides the core functionality for Kafka operations including producers, consumers,
//! configuration management, and message handling.
//!
//! ## Overview
//!
//! The core module is designed to provide a high-level, type-safe interface for Kafka operations
//! while maintaining flexibility and performance. It includes:
//!
//! - **KafkaProducer**: Async producer for sending messages to Kafka topics
//! - **KafkaConsumer**: Async consumer with concurrency support for processing messages
//! - **KafkaClientConfig**: Flexible configuration management using HashMap-based approach
//! - **Message Extensions**: Utilities for message parsing, serialization, and handling
//! - **Error Handling**: Comprehensive error types and handling mechanisms
//!
//! ## Quick Start
//!
//! ```rust
//! use rust_common::kafka::core::*;
//! use rdkafka::config::RDKafkaLogLevel;
//!
//! // Create configuration
//! let config = KafkaClientConfig::new(
//!     "my-cluster".to_string(),
//!     Some(vec!["my-topic".to_string()]),
//!     RDKafkaLogLevel::Info,
//! );
//!
//! // Create producer
//! let producer = KafkaProducer::new(config.clone())?;
//!
//! // Send message
//! #[derive(serde::Serialize, Debug)]
//! struct MyMessage {
//!     content: String,
//!     timestamp: u64,
//! }
//!
//! let message = MyMessage {
//!     content: "Hello Kafka!".to_string(),
//!     timestamp: 1234567890,
//! };
//!
//! producer.send(message, "my-topic").await?;
//!
//! // Create consumer
//! let consumer = KafkaConsumer::new(config, Some(10))?;
//!
//! // Process messages
//! consumer.start(|msg| async move {
//!     let payload = String::from_utf8_lossy(msg.payload().unwrap_or_default());
//!     println!("Received: {}", payload);
//!     Ok(())
//! }).await?;
//! ```
//!
//! ## Features
//!
//! - **Async/Await Support**: Full async support for non-blocking operations
//! - **Type Safety**: Strong typing with generic constraints
//! - **Concurrency**: Built-in concurrency support for message processing
//! - **Error Handling**: Comprehensive error types with proper propagation
//! - **Configuration**: Flexible configuration system similar to rdkafka
//! - **Logging**: Integrated logging with tracing
//! - **Message Parsing**: Built-in JSON message parsing with error handling
//!
//! ## Error Handling
//!
//! The module provides detailed error types:
//!
//! ```rust
//! use rust_common::kafka::core::KafkaError;
//!
//! match result {
//!     Ok(_) => println!("Success"),
//!     Err(e) => {
//!         if let Some(kafka_error) = e.downcast_ref::<KafkaError>() {
//!             match kafka_error {
//!                 KafkaError::TimeoutError(msg) => println!("Timeout: {}", msg),
//!                 KafkaError::SerializationError(msg) => println!("Serialization error: {}", msg),
//!                 KafkaError::ConnectionError(msg) => println!("Connection error: {}", msg),
//!                 _ => println!("Other error: {:?}", kafka_error),
//!             }
//!         }
//!     }
//! }
//! ```

pub mod error;
pub mod extensions;
pub mod kafka_config;
pub mod kafka_consumer;
pub mod kafka_producer;

pub use error::*;
pub use extensions::*;
pub use kafka_config::*;
pub use kafka_consumer::*;
pub use kafka_producer::*;
