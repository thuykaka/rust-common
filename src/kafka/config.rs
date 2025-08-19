#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub cluster_id: String,
    pub bootstrap_servers: String,
    pub client_id: Option<String>,
    pub topics: Vec<String>,
}
