use std::{
    fmt::{Debug, Formatter},
    time::Duration,
};

use base64::{engine::general_purpose, Engine as _};
use eyre::WrapErr;
use rdkafka::{
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord, Producer},
    ClientConfig,
};

use super::{KafkaConfig, Message, StreamingClient};

use crate::{Result, Sensitive};

const NO_RETRY: Duration = Duration::from_secs(0);

#[derive(Clone)]
pub struct KafkaClient {
    producer: FutureProducer,
    health_check_topic: String,
}

impl KafkaClient {
    /// Creates a Kafka client connected to the broker.
    ///
    /// The connection is validated immediately after creation, and if not connected, the
    /// client creation will fail with an error.
    pub async fn new(config: &KafkaConfig) -> Result<Self> {
        tracing::info!(config = ?config, "initing kafka-client");

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", &config.kafka_url);

        if let (Some(key), Some(certificate), Some(ca)) =
            (&config.kafka_key, &config.kafka_cert, &config.kafka_ca)
        {
            client_config
                .set("security.protocol", "ssl")
                .set("ssl.key.pem", pem_string_from_base64(key)?.0)
                .set(
                    "ssl.certificate.pem",
                    pem_string_from_base64(certificate)?.0,
                )
                .set("ssl.ca.pem", pem_string_from_base64(ca)?.0);
        }

        let client = KafkaClient {
            producer: client_config
                .create()
                .wrap_err("Failed to open connection with Kafka")?,
            health_check_topic: config.kafka_health_check_topic.clone(),
        };

        client.health_check().await?;

        Ok(client)
    }
}

fn pem_string_from_base64(base64: &Sensitive<String>) -> Result<Sensitive<String>> {
    let pem_bytes = &general_purpose::STANDARD.decode(&base64.0)?;

    let pem_text = std::str::from_utf8(pem_bytes.as_slice())?;
    Ok(Sensitive::from(pem_text.to_string()))
}

#[crate::async_trait]
impl StreamingClient for KafkaClient {
    /// Publishes a pre-defined Kafka message to the broker.
    async fn publish(&self, message: Message) -> Result<()> {
        // convert headers
        let mut kafka_headers = OwnedHeaders::new_with_capacity(message.headers.len());
        for (key, value) in message.headers.into_iter() {
            kafka_headers = kafka_headers.insert(Header {
                key: &key,
                value: Some(&value),
            });
        }

        // convert entire message
        let kafka_record = FutureRecord::to(&message.topic)
            .key(&message.key)
            .payload(&message.payload)
            .headers(kafka_headers);

        // publish and parse Kafka complex result
        self.producer
            .send(kafka_record, NO_RETRY)
            .await
            .map_err(|e| e.0)
            .wrap_err("Failed to send message to Kafka")?;
        Ok(())
    }

    async fn health_check(&self) -> Result<()> {
        self.producer
            .client()
            .fetch_metadata(Some(&self.health_check_topic), Duration::from_millis(500))
            .wrap_err("Failed to check Kafka health")?;
        Ok(())
    }
}

impl Debug for KafkaClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("producer", &"...")
            .finish_non_exhaustive()
    }
}
