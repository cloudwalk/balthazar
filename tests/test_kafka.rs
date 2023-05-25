use std::collections::HashMap;

use balthazar::{KafkaClient, KafkaConfig, Message, StreamExt, StreamingClient};

use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message as KafkaMessage, Offset, TopicPartitionList,
};

#[tokio::test]
async fn test_pushing_on_kafka_local_client() {
    let test_topic = "topic";
    let config = KafkaConfig {
        kafka_url: "0.0.0.0:29092".to_string(),
        kafka_ca: None,
        kafka_cert: None,
        kafka_key: None,
        kafka_health_check_topic: test_topic.to_string(),
    };

    let kafka_client: KafkaClient = KafkaClient::new(&config).await.unwrap();
    let uuid = "16d5d5c9-6243-450c-9851-e5dca855552d".to_string();

    let kafka_message = Message {
        headers: HashMap::new(),
        key: uuid.clone(),
        payload: uuid.clone(),
        topic: test_topic.to_string(),
    };

    let result = kafka_client.publish(kafka_message).await;
    assert!(result.is_ok());

    let kafka_message = get_message_from_kafka(test_topic).await.unwrap();

    assert_eq!(uuid.to_string(), kafka_message);
}

async fn get_message_from_kafka(topic: &str) -> balthazar::Result<String> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "test_consumer_group")
        .set("bootstrap.servers", "0.0.0.0:29092")
        .set("session.timeout.ms", "6000")
        .create()
        .expect("Consumer creation failed");

    let mut topics = TopicPartitionList::new();
    topics
        .add_partition_offset(topic, 0, Offset::OffsetTail(1))
        .unwrap();
    consumer.assign(&topics).unwrap();

    let last_message = consumer.stream().next().await.unwrap()?;

    let parsed_payload = last_message.payload_view::<str>().unwrap()?;

    Ok(parsed_payload.to_string())
}
