use crate::ratelimit::{RatelimitedRouteResponse, Ratelimiter};
use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;
use todel::http::ClientIP;
use todel::models::{ErrorResponse, ErrorResponseData, Message, Payload, ValidationError};
use todel::Conf;

#[post("/", data = "<message>")]
pub async fn index(
    message: Json<Message>,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RatelimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut ratelimiter = Ratelimiter::new("message_create", address, conf.inner());
    ratelimiter.process_ratelimit(&mut cache).await?;
    let message = message.into_inner();
    if message.author.len() < 2 || message.author.len() > 32 {
        ratelimiter.wrap_response(Err(ValidationError {
            field_name: "author".to_string(),
            error: "Message author has to be between 2 and 32 characters long.".to_string(),
        }
        .to_error_response()))
    } else if message.content.is_empty() || message.content.len() > conf.oprish.message_limit {
        ratelimiter.wrap_response(Err(ValidationError {
            field_name: "content".to_string(),
            error: format!(
                "Message content has to be between 1 and {} characters long.",
                conf.oprish.message_limit
            ),
        }
        .to_error_response()))
    } else {
        let payload = Payload::MessageCreate(message);
        cache
            .publish::<&str, String, ()>("oprish-events", serde_json::to_string(&payload).unwrap())
            .await
            .unwrap();
        if let Payload::MessageCreate(message) = payload {
            ratelimiter.wrap_response(Ok(Json(message)))
        } else {
            unreachable!()
        }
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
