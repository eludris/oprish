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
use todel::Conf;

#[derive(Database)]
#[database("redis-cache")]
pub struct Cache(deadpool_redis::Pool);

#[launch]
fn rocket() -> Rocket<Build> {
    #[cfg(tests)]
    {
        use std::env;
        env::set_var("ELUDRIS_CONF", "tests/Eludris.toml");
    }
    dotenv::dotenv().ok();
    env_logger::try_init().ok();

    let conf = Conf::new_from_env();

    rocket::build()
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes())
        .manage(conf)
        .attach(Cache::init())
        .attach(cors::Cors)
}
