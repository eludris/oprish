#[cfg(test)]
mod tests {
    use crate::{rocket, Cache};
    use deadpool_redis::Connection;
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use todel::{
        models::{Info, InstanceRatelimits, Message},
        Conf,
    };

    #[rocket::async_test]
    async fn index() {
        let client = Client::untracked(rocket()).await.unwrap();
        let response = client.get("/").dispatch().await;
        let conf = &client.rocket().state::<Conf>().unwrap();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&Info {
                instance_name: conf.instance_name.clone(),
                description: conf.description.clone(),
            })
            .unwrap()
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

    #[rocket::async_test]
    async fn ratelimits() {
        let client = Client::untracked(rocket()).await.unwrap();
        let response = client.get("/ratelimits").dispatch().await;
        let conf = &client.rocket().state::<Conf>().unwrap();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceRatelimits {
                oprish: conf.oprish.ratelimits.clone(),
                pandemonium: conf.pandemonium.ratelimit.clone(),
                effis: conf.effis.ratelimit.clone(),
            })
            .unwrap()
        )
    }
}
