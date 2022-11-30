pub mod messages;
pub mod ratelimits;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{http::ClientIP, models::InstanceInfo, Conf};

use crate::{
    ratelimit::{RatelimitedRouteResponse, Ratelimiter},
    Cache,
}; // poggers

#[get("/")]
pub async fn index(
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RatelimitedRouteResponse<Json<InstanceInfo>> {
    let mut ratelimiter = Ratelimiter::new("info", address, conf.inner());
    ratelimiter.process_ratelimit(&mut cache).await?;
    ratelimiter.wrap_response(Json(InstanceInfo {
        instance_name: conf.instance_name.clone(),
        description: conf.description.clone(),
        message_limit: conf.oprish.message_limit,
        oprish_url: conf.oprish.url.clone(),
        pandemonium_url: conf.pandemonium.url.clone(),
        effis_url: conf.effis.url.clone(),
    }))
}

pub fn get_routes() -> Vec<Route> {
    routes![index, ratelimits::ratelimits]
}
