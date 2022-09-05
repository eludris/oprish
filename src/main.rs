#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;

mod producer;
mod routes;

use crate::producer::Producer;
use rocket::{Build, Rocket};
use routes::*;
use std::env;
use todel::models::{Feature, Info};

#[launch]
fn rocket() -> Rocket<Build> {
    #[cfg(test)]
    {
        env::set_var("INSTANCE_NAME", "WooChat")
    }
    let _ = env_logger::try_init();

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
        .mount("/", routes![index])
        .mount("/messages", messages::get_routes())
        .manage(info)
        .manage(producer)
}
