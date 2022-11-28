#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;

mod cors;
mod ratelimit;
mod routes;

use std::env;

use rocket::{Build, Config, Rocket};
use rocket_db_pools::Database;
use routes::*;
use todel::Conf;

#[derive(Database)]
#[database("cache")]
pub struct Cache(deadpool_redis::Pool);

#[launch]
fn rocket() -> Rocket<Build> {
    #[cfg(test)]
    {
        use std::env;
        env::set_var("ELUDRIS_CONF", "tests/Eludris.toml");
    }
    dotenv::dotenv().ok();
    env_logger::try_init().ok();

    let conf = Conf::new_from_env();

    let config = Config::figment()
        .merge((
            "databases.db",
            rocket_db_pools::Config {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "mysql://root:root@localhost:3306/eludris".to_string()),
                min_connections: None,
                max_connections: 1024,
                connect_timeout: 3,
                idle_timeout: None,
            },
        ))
        .merge((
            "databases.cache",
            rocket_db_pools::Config {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
                min_connections: None,
                max_connections: 1024,
                connect_timeout: 3,
                idle_timeout: None,
            },
        ));

    rocket::custom(config)
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes())
        .manage(conf)
        .attach(Cache::init())
        .attach(cors::Cors)
}
