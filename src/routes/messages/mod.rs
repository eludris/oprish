use crate::ratelimit::Ratelimiter;
use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;
use todel::models::Message;
use todel::oprish::{
    ClientIP, ErrorResponse, ErrorResponseData, RatelimitedRoutResponse, Response,
};
use todel::Conf;

#[post("/", data = "<message>")]
pub async fn index(
    message: Json<Message>,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RatelimitedRoutResponse<Json<Response<Message>>> {
    let mut ratelimiter = Ratelimiter::new("message_create", address, conf.inner());
    ratelimiter.process_ratelimit(&mut cache).await?;
    let message = message.into_inner();
    if message.author.len() < 2 || message.author.len() > 32 {
        Ok(
            ratelimiter.wrap_response(Json(Response::Failure(ErrorResponse::new(
                ErrorResponseData::ValidationError {
                    invalid_key: "author".to_string(),
                    info: "Message author has to be between 2 and 32 characters long.".to_string(),
                },
            )))),
        )
    } else if message.content.is_empty() || message.content.len() > conf.oprish.message_limit {
        Ok(
            ratelimiter.wrap_response(Json(Response::Failure(ErrorResponse::new(
                ErrorResponseData::ValidationError {
                    invalid_key: "content".to_string(),
                    info: format!(
                        "Message content has to be between 1 and {} characters long.",
                        conf.oprish.message_limit
                    ),
                },
            )))),
        )
    } else {
        cache
            .publish::<&str, String, ()>("oprish-events", serde_json::to_string(&message).unwrap())
            .await
            .unwrap();
        Ok(ratelimiter.wrap_response(Json(Response::Success(message))))
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
