#[cfg(test)]
mod tests {
    use crate::{rocket, Cache};
    use deadpool_redis::Connection;
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use todel::{
        models::{InstanceInfo, InstanceRatelimits, Message, Payload},
        Conf,
    };

    #[rocket::async_test]
    async fn index() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let response = client.get("/").dispatch().await;
        let conf = &client.rocket().state::<Conf>().unwrap();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceInfo {
                instance_name: conf.instance_name.clone(),
                description: conf.description.clone(),
                message_limit: conf.oprish.message_limit,
                oprish_url: conf.oprish.url.clone(),
                pandemonium_url: conf.pandemonium.url.clone(),
                effis_url: conf.effis.url.clone(),
            })
            .unwrap()
        )
    }

    #[rocket::async_test]
    async fn send_message() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let message = Message {
            author: "Woo".to_string(),
            content: "HeWoo there".to_string(),
        };
        let message_str = serde_json::to_string(&message).unwrap();
        let payload = serde_json::to_string(&Payload::MessageCreate(message)).unwrap();

        let pool = client.rocket().state::<Cache>().unwrap();

        let cache = pool.get().await.unwrap();
        let cache = Connection::take(cache);
        let mut cache = cache.into_pubsub();
        cache.subscribe("oprish-events").await.unwrap();

        let response = client
            .post("/messages/")
            .body(&message_str)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), message_str);

        assert_eq!(
            cache
                .into_on_message()
                .next()
                .await
                .unwrap()
                .get_payload::<String>()
                .unwrap(),
            payload
        );
    }

    #[rocket::async_test]
    async fn ratelimits() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let response = client.get("/ratelimits").dispatch().await;
        let conf = &client.rocket().state::<Conf>().unwrap();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceRatelimits {
                oprish: conf.oprish.ratelimits.clone(),
                pandemonium: conf.pandemonium.ratelimit.clone(),
                effis: conf.effis.ratelimits.clone(),
            })
            .unwrap()
        )
    }
}
