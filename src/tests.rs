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

        let response = client
            .post("/messages/")
            .body(message.clone())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), message);
    }
}
