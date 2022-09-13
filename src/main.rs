#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;

mod producer;
mod ratelimit;
mod routes;

use crate::producer::Producer;
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
    let brokers = env::var("BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let topic = env::var("TOPIC").unwrap_or_else(|_| "oprish".to_string());

    let features = vec![Feature {
        id: 0,
        name: "base".to_string(),
    }];
    let info = Info {
        instance_name,
        features,
    };

    let producer = Producer::new(brokers, topic, "oprish".to_string());

    rocket::build()
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes())
        .manage(info)
        .manage(producer)
        .attach(Cache::init())
}
