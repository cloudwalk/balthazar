use crate::{Parser, Sensitive};

#[derive(Debug, Clone, Parser)]
pub struct KafkaConfig {
    #[clap(long = "kafka-url", env = "KAFKA_URL")]
    pub kafka_url: String,

    #[clap(long = "kafka-url", env = "KAFKA_HEALTH_CHECK_TOPIC")]
    pub kafka_health_check_topic: String,

    #[clap(long = "kafka-key", env = "KAFKA_KEY")]
    pub kafka_key: Option<Sensitive<String>>,

    #[clap(long = "kafka-cert", env = "KAFKA_CERT")]
    pub kafka_cert: Option<Sensitive<String>>,

    #[clap(long = "kafka-ca", env = "KAFKA_CA")]
    pub kafka_ca: Option<Sensitive<String>>,
}
