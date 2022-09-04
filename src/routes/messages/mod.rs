use crate::producer::Producer;
use rocket::serde::json::Json;
use rocket::{Route, State};
use todel::models::Message;

#[post("/", data = "<message>")]
pub async fn index(message: Json<Message>, producer: &State<Producer>) -> Json<Message> {
    let message = message.into_inner();
    producer.send(&message).await;
    Json(message)
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
