use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{http::ClientIP, models::InstanceRatelimits, Conf};

use crate::{
    ratelimit::{RatelimitedRouteResponse, Ratelimiter},
    Cache,
};

#[get("/ratelimits")]
pub async fn ratelimits(
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RatelimitedRouteResponse<Json<InstanceRatelimits>> {
    let conf = conf.inner();
    let mut ratelimiter = Ratelimiter::new("ratelimits", address, conf);
    ratelimiter.process_ratelimit(&mut cache).await?;
    ratelimiter.wrap_response(Json(InstanceRatelimits {
        oprish: conf.oprish.ratelimits.clone(),
        pandemonium: conf.pandemonium.ratelimit.clone(),
        effis: conf.effis.ratelimits.clone(),
    }))
}
