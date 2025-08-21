use std::collections::HashMap;

use rdkafka::{config::RDKafkaLogLevel, ClientConfig};
use uuid::Uuid;

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
    pub fn new(cluster_id: String, bootstrap_servers: String) -> Self {
        let mut conf_map = HashMap::new();
        let client_id = format!("{}-{}", cluster_id.clone(), Uuid::new_v4());

        conf_map.insert("client.id".to_string(), client_id);
        conf_map.insert("bootstrap.servers".to_string(), bootstrap_servers);
        conf_map.insert("allow.auto.create.topics".to_string(), "true".to_string());

        Self {
            cluster_id,
            topics: None,
            conf_map,
            log_level: RDKafkaLogLevel::Info,
        }
    }

    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics = Some(topics);
        self
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
