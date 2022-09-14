use std::time::Duration;

use crate::producer::Producer;
use crate::ratelimit::Ratelimiter;
use crate::Cache;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;
use serde_valid::Validate;
use todel::models::Message;
use todel::oprish::{ClientIP, MessageCreateResponse, RatelimitedRoutResponse};

#[post("/", data = "<message>")]
pub async fn index(
    message: Json<Message>,
    producer: &State<Producer>,
    address: ClientIP,
    cache: Connection<Cache>,
) -> RatelimitedRoutResponse<(Status, Json<MessageCreateResponse>)> {
    let mut ratelimiter =
        Ratelimiter::new(cache, "message_send", address, Duration::from_secs(5), 10);
    ratelimiter.process_ratelimit().await?;
    let message = message.into_inner();
    if let Err(err) = message.validate() {
        Ok(ratelimiter.wrap_response((
            Status::BadRequest,
            Json(MessageCreateResponse::ValidationError(err)),
        )))
    } else {
        producer.send(&message).await;
        Ok(ratelimiter.wrap_response((Status::Ok, Json(MessageCreateResponse::Sucess(message)))))
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
