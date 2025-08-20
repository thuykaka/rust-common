use std::collections::HashMap;

use rdkafka::{config::RDKafkaLogLevel, ClientConfig};
use uuid::Uuid;

/// Kafka client configuration for producers and consumers
///
/// This struct provides a flexible configuration system for Kafka clients,
/// using a HashMap-based approach similar to rdkafka's ClientConfig.
///
/// # Features
///
/// - HashMap-based configuration (similar to rdkafka)
/// - Builder pattern for easy configuration
/// - Automatic UUID generation for client IDs
/// - Default values for common settings
/// - Conversion to rdkafka ClientConfig
///
/// # Example
///
/// ```rust
/// use rust_common::kafka::core::*;
/// use rdkafka::config::RDKafkaLogLevel;
///
/// // Basic configuration
/// let config = KafkaClientConfig::new(
///     "my-cluster".to_string(),
///     Some(vec!["topic1".to_string(), "topic2".to_string()]),
///     RDKafkaLogLevel::Info,
/// );
///
/// // Advanced configuration with custom settings
/// let mut config = KafkaClientConfig::new(
///     "my-cluster".to_string(),
///     None,
///     RDKafkaLogLevel::Debug,
/// );
///
/// config
///     .set("bootstrap.servers", "localhost:9092")
///     .set("compression.type", "gzip")
///     .set("batch.size", "16384")
///     .set("linger.ms", "100");
///
/// // Convert to rdkafka ClientConfig
/// let client_config = config.to_client_config();
/// ```
///
/// # Default Values
///
/// The following default values are set automatically:
///
/// - `group.id` = cluster_id
/// - `client.id` = cluster_id-uuid
/// - `allow.auto.create.topics` = "true"
///
/// # Thread Safety
///
/// `KafkaClientConfig` is thread-safe and can be cloned for use across multiple threads.
#[derive(Debug, Clone)]
pub struct KafkaClientConfig {
    /// The cluster identifier used for group.id and client.id generation
    pub cluster_id: String,
    /// Optional list of topics to subscribe to (for consumers)
    pub topics: Option<Vec<String>>,
    /// Configuration map containing all Kafka client settings
    pub conf_map: HashMap<String, String>,
    /// Log level for rdkafka logging
    pub log_level: RDKafkaLogLevel,
}

impl KafkaClientConfig {
    pub fn new(cluster_id: String, bootstrap_servers: String, topics: Option<Vec<String>>) -> Self {
        let mut conf_map = HashMap::new();
        let client_id = format!("{}-{}", cluster_id.clone(), Uuid::new_v4());

        conf_map.insert("client.id".to_string(), client_id);
        conf_map.insert("bootstrap.servers".to_string(), bootstrap_servers);
        conf_map.insert("allow.auto.create.topics".to_string(), "true".to_string());

        Self {
            cluster_id,
            topics,
            conf_map,
            log_level: RDKafkaLogLevel::Info,
        }
    }

    pub fn set<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.conf_map.insert(key.into(), value.into());
        self
    }

    pub fn set_log_level(&mut self, log_level: RDKafkaLogLevel) -> &mut Self {
        self.log_level = log_level;
        self
    }

    pub fn get_conf_map(&self) -> HashMap<String, String> {
        self.conf_map.clone()
    }

    pub fn get_conf_with_key(&self, key: &str) -> Option<&str> {
        self.conf_map.get(key).map(|v| v.as_str())
    }

    pub fn get_cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub fn get_topics(&self) -> Option<Vec<String>> {
        self.topics.clone()
    }

    pub fn to_client_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();
        for (key, value) in &self.conf_map {
            config.set(key, value);
        }
        config.set_log_level(self.log_level);
        config
    }
}
