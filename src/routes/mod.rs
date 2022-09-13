pub mod messages;

use rocket::{serde::json::Json, Route, State};
use todel::models::Info; // poggers

#[get("/")]
pub async fn index(info: &State<Info>) -> Json<&Info> {
    Json(info.inner())
}

pub fn get_routes() -> Vec<Route> {
    routes![index]
}
