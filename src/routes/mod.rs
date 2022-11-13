pub mod messages;
pub mod ratelimits;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{
    models::Info,
    oprish::{ClientIP, RatelimitedRoutResponse},
    Conf,
};

use crate::{ratelimit::Ratelimiter, Cache}; // poggers

#[get("/")]
pub async fn index(
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RatelimitedRoutResponse<Json<Info>> {
    let mut ratelimiter = Ratelimiter::new("info", address, conf.inner());
    ratelimiter.process_ratelimit(&mut cache).await?;
    Ok(ratelimiter.wrap_response(Json(Info {
        instance_name: conf.instance_name.clone(),
        description: conf.description.clone(),
    })))
}

pub fn get_routes() -> Vec<Route> {
    routes![index, ratelimits::ratelimits]
}
