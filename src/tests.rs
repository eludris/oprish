#[cfg(test)]
mod tests {
    use std::env;

    use rdkafka::{
        consumer::{Consumer, StreamConsumer},
        ClientConfig, Message as KafkaMessage,
    };
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use todel::models::{Info, Message};

    use crate::rocket;

    #[rocket::async_test]
    async fn index() {
        let client = Client::untracked(rocket()).await.unwrap();
        let response = client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&client.rocket().state::<Info>().unwrap()).unwrap()
        )
    }

    #[rocket::async_test]
    async fn send_message() {
        let client = Client::untracked(rocket()).await.unwrap();
        let message = serde_json::to_string(&Message {
            author: "Woo".to_string(),
            content: "HeWoo there".to_string(),
        })
        .unwrap();

        let brokers = env::var("BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let topic = env::var("TOPIC").unwrap_or_else(|_| "oprish".to_string());

        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", "oprish-test")
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .create()
            .unwrap();

        consumer.subscribe(&[&topic]).unwrap();

        let response = client
            .post("/messages/")
            .body(message.clone())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), message);

        assert_eq!(
            String::from_utf8(
                consumer
                    .stream()
                    .next()
                    .await
                    .unwrap()
                    .unwrap()
                    .payload()
                    .unwrap()
                    .to_vec(),
            )
            .unwrap(),
            message
        );
    }
}
