use crate::ratelimit::Ratelimiter;
use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket_db_pools::Connection;
use serde::Serialize;
use serde_valid::validation::Errors;
use serde_valid::Validate;
use std::time::Duration;
use todel::models::Message;
use todel::oprish::{ClientIP, RatelimitedRoutResponse};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MessageCreateResponse {
    Success(Message),
    ValidationError(Errors),
}

#[post("/", data = "<message>")]
pub async fn index(
    message: Json<Message>,
    address: ClientIP,
    mut cache: Connection<Cache>,
) -> RatelimitedRoutResponse<(Status, Json<MessageCreateResponse>)> {
    let mut ratelimiter = Ratelimiter::new("message_send", address, Duration::from_secs(5), 10);
    ratelimiter.process_ratelimit(&mut cache).await?;
    let message = message.into_inner();
    if let Err(err) = message.validate() {
        Ok(ratelimiter.wrap_response((
            Status::BadRequest,
            Json(MessageCreateResponse::ValidationError(err)),
        )))
    } else {
        cache
            .publish::<&str, String, ()>("oprish-events", serde_json::to_string(&message).unwrap())
            .await
            .unwrap();
        Ok(ratelimiter.wrap_response((Status::Ok, Json(MessageCreateResponse::Success(message)))))
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
