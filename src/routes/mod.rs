pub mod messages;

use rocket::{serde::json::Json, State};
use todel::models::Info; // poggers

#[get("/")]
pub async fn index(info: &State<Info>) -> Json<&Info> {
    Json(info.inner())
}
