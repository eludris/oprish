use std::time::Duration;

use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use serde::Serialize;

/// A simple abstraction for a one-topic and one-key Kafka producer.
pub struct Producer {
    producer: FutureProducer,
    topic: String,
    key: String,
}

impl Producer {
    pub fn new(brokers: String, topic: String, key: String) -> Producer {
        Producer {
            producer: ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .expect("Failed to create the Kafka producer"),
            topic,
            key,
        }
    }

    pub async fn send<T: Serialize>(&self, message: T) {
        match serde_json::to_string(&message) {
            Ok(payload) => {
                self.producer
                    .send(
                        FutureRecord::to(&self.topic)
                            .key(&self.key)
                            .payload(&payload),
                        Duration::from_secs(0),
                    )
                    .await
                    .expect("Failed to send message to Kafka");
            }
            Err(err) => log::warn!("Failed to convert message to json: {}", err),
        };
    }
}
