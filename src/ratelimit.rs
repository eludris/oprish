use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::http::Header;
use rocket_db_pools::Connection;
use todel::{
    models::{ErrorResponse, ErrorResponseData, RatelimitError},
    Conf,
};

pub type RatelimitedRouteResponse<T> =
    Result<RatelimitHeaderWrapper<T>, RatelimitHeaderWrapper<ErrorResponse>>;

/// The necessary headers for responses
#[derive(Debug, Responder)]
#[response(content_type = "json")]
pub struct RatelimitHeaderWrapper<T> {
    pub inner: T,
    pub ratelimit_reset: Header<'static>,
    pub ratelimit_max: Header<'static>,
    pub ratelimit_last_reset: Header<'static>,
    pub ratelimit_request_count: Header<'static>,
}

// Can derive debug :chad:
/// A simple Ratelimiter than can keep track of ratelimit data from KeyDB and add ratelimit
/// related headers to a response type
#[derive(Debug)]
pub struct Ratelimiter {
    key: String,
    reset_after: Duration,
    request_limit: u32,
    request_count: u32,
    last_reset: u64,
}

impl Ratelimiter {
    /// Creates a new Ratelimiter
    pub fn new<I>(bucket: &str, identifier: I, conf: &Conf) -> Ratelimiter
    where
        I: Display,
    {
        let ratelimit = match bucket {
            "info" => &conf.oprish.ratelimits.info,
            "message_create" => &conf.oprish.ratelimits.message_create,
            "ratelimits" => &conf.oprish.ratelimits.ratelimits,
            _ => unreachable!(),
        };
        Ratelimiter {
            key: format!("ratelimit:{}:{}", identifier, bucket),
            reset_after: Duration::from_secs(ratelimit.reset_after as u64),
            request_limit: ratelimit.limit,
            request_count: 0,
            last_reset: 0,
        }
    }

    /// Checks if a bucket is ratelimited, if so returns an Error with an ErrorResponse
    pub async fn process_ratelimit(
        &mut self,
        cache: &mut Connection<Cache>,
    ) -> Result<(), RatelimitHeaderWrapper<ErrorResponse>> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        if let (Some(last_reset), Some(request_count)) = cache
            .hget::<&str, (&str, &str), (Option<u64>, Option<u32>)>(
                &self.key,
                ("last_reset", "request_count"),
            )
            .await
            .expect("Couldn't query cache")
        {
            self.last_reset = last_reset;
            self.request_count = request_count;
            if now - self.last_reset >= self.reset_after.as_millis() as u64 {
                cache
                    .del::<&str, ()>(&self.key)
                    .await
                    .expect("Couldn't query cache");
                cache
                    .hset_multiple::<&str, &str, u64, ()>(
                        &self.key,
                        &[("last_reset", now), ("request_count", 0)],
                    )
                    .await
                    .expect("Couldn't query cache");
                self.last_reset = now;
                self.request_count = 0;
                log::debug!("Reset bucket for {}", self.key);
            }
            if self.request_count >= self.request_limit {
                log::info!("Ratelimited bucket {}", self.key);
                Err(self
                    .wrap_response(
                        RatelimitError {
                            retry_after: self.last_reset + self.reset_after.as_millis() as u64
                                - now,
                        }
                        .to_error_response(),
                    )
                    .unwrap())
            } else {
                cache
                    .hincr::<&str, &str, u8, ()>(&self.key, "request_count", 1)
                    .await
                    .expect("Couldn't query cache");
                self.request_count += 1;
                Ok(())
            }
        } else {
            log::debug!("New bucket for {}", self.key);
            cache
                .hset_multiple::<&str, &str, u64, ()>(
                    &self.key,
                    &[("last_reset", now), ("request_count", 1)],
                )
                .await
                .expect("Couldn't query cache");
            Ok(())
        }
    }

    /// Wraps a response in a RatelimitHeaderWrapper which adds headers relavent to ratelimiting
    pub fn wrap_response<R>(&self, data: R) -> RatelimitedRouteResponse<R> {
        Ok(RatelimitHeaderWrapper {
            inner: data,
            ratelimit_reset: Header::new(
                "X-Ratelimit-Reset",
                self.reset_after.as_millis().to_string(),
            ),
            ratelimit_max: Header::new("X-Ratelimit-Max", self.request_limit.to_string()),
            ratelimit_last_reset: Header::new(
                "X-Ratelimit-Last-Reset",
                self.last_reset.to_string(),
            ),
            ratelimit_request_count: Header::new(
                "X-Ratelimit-Request-Count",
                self.request_count.to_string(),
            ),
        })
    }
}
