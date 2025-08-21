use std::collections::HashMap;

use rdkafka::{config::RDKafkaLogLevel, ClientConfig};
use uuid::Uuid;

/// KafkaClientConfig holds the configuration settings for Kafka clients.
/// It provides methods to customize settings for producers and consumers.
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
    /// Creates a new KafkaClientConfig with the specified cluster ID and bootstrap servers.
    ///
    /// # Arguments
    ///
    /// * `cluster_id` - A unique identifier for the Kafka cluster.
    /// * `bootstrap_servers` - A comma-separated list of Kafka brokers.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of KafkaClientConfig.
    pub fn new(cluster_id: String, bootstrap_servers: String) -> Self {
        let mut conf_map = HashMap::new();
        let client_id = format!("{}-{}", cluster_id.clone(), Uuid::new_v4());

        conf_map.insert("client.id".to_string(), client_id);
        conf_map.insert("bootstrap.servers".to_string(), bootstrap_servers);
        conf_map.insert("allow.auto.create.topics".to_string(), "true".to_string());
        conf_map.insert("message.max.bytes".to_string(), "1000000000".to_string());
        conf_map.insert("retry.backoff.ms".to_string(), "200".to_string());

        Self {
            cluster_id,
            topics: None,
            conf_map,
            log_level: RDKafkaLogLevel::Info,
        }
    }

    /// Sets the topics for the consumer to subscribe to.
    ///
    /// # Arguments
    ///
    /// * `topics` - A vector of topic names.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated KafkaClientConfig instance.
    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics = Some(topics);
        self
    }

    /// Sets a custom configuration key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key.
    /// * `value` - The configuration value.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated KafkaClientConfig instance.
    pub fn set<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.conf_map.insert(key.into(), value.into());
        self
    }

    /// Sets the log level for rdkafka logging.
    ///
    /// # Arguments
    ///
    /// * `log_level` - The desired log level.
    ///
    /// # Returns
    ///
    /// * `&mut Self` - The updated KafkaClientConfig instance.
    pub fn set_log_level(&mut self, log_level: RDKafkaLogLevel) -> &mut Self {
        self.log_level = log_level;
        self
    }

    /// Retrieves the configuration map.
    ///
    /// # Returns
    ///
    /// * `HashMap<String, String>` - A clone of the configuration map.
    pub fn get_conf_map(&self) -> HashMap<String, String> {
        self.conf_map.clone()
    }

    /// Retrieves a configuration value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key.
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The configuration value if it exists.
    pub fn get_conf_with_key(&self, key: &str) -> Option<&str> {
        self.conf_map.get(key).map(|v| v.as_str())
    }

    /// Retrieves the cluster ID.
    ///
    /// # Returns
    ///
    /// * `&str` - The cluster ID.
    pub fn get_cluster_id(&self) -> &str {
        &self.cluster_id
    }

    /// Retrieves the list of topics.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<String>>` - A clone of the topics vector if it exists.
    pub fn get_topics(&self) -> Option<Vec<String>> {
        self.topics.clone()
    }

    /// Converts the configuration to a rdkafka ClientConfig.
    ///
    /// # Returns
    ///
    /// * `ClientConfig` - The rdkafka ClientConfig instance.
    pub fn to_client_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();
        for (key, value) in &self.conf_map {
            config.set(key, value);
        }
        config.set_log_level(self.log_level);
        config
    }
}
