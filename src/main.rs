#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;

mod cors;
mod ratelimit;
mod routes;

use rocket::{Build, Rocket};
use rocket_db_pools::Database;
use routes::*;
use std::env;
use todel::models::{Feature, Info};

#[derive(Database)]
#[database("redis-cache")]
pub struct Cache(deadpool_redis::Pool);

#[launch]
fn rocket() -> Rocket<Build> {
    #[cfg(test)]
    {
        env::set_var("INSTANCE_NAME", "WooChat")
    }
    dotenv::dotenv().ok();
    env_logger::try_init().ok();

    let instance_name =
        env::var("INSTANCE_NAME").expect("Can't find \"INSTANCE_NAME\" environment variable");

    let features = vec![Feature {
        id: 0,
        name: "base".to_string(),
    }];
    let info = Info {
        instance_name,
        features,
    };

    rocket::build()
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes())
        .manage(info)
        .attach(Cache::init())
        .attach(cors::Cors)
}
