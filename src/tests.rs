#[cfg(test)]
mod tests {
    use crate::{rocket, Cache};
    use deadpool_redis::Connection;
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use todel::models::{Info, Message};

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

        let pool = client.rocket().state::<Cache>().unwrap();

        let cache = pool.get().await.unwrap();
        let cache = Connection::take(cache);
        let mut cache = cache.into_pubsub();
        cache.subscribe("oprish-events").await.unwrap();

        let response = client
            .post("/messages/")
            .body(message.clone())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), message);

        assert_eq!(
            cache
                .into_on_message()
                .next()
                .await
                .unwrap()
                .get_payload::<String>()
                .unwrap(),
            message
        );
    }
}
