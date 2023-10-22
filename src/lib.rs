#![allow(clippy::needless_doctest_main)]
//! **rocket-sentry** is a simple add-on for the **Rocket** web framework to simplify
//! integration with the **Sentry** application monitoring system.
//!
//! Or maybe...
//!
//! > "The Rocket Sentry is a static rocket-firing gun platform that is based on a
//! > Personality Construct and used in the Aperture Science Enrichment Center."
//! >
//! > -- [Half-Life wiki](https://half-life.fandom.com/wiki/Rocket_Sentry)
//!
//! Example usage
//! =============
//!
//! ```no_run
//! # #[macro_use]
//! # extern crate rocket;
//! use rocket_sentry::RocketSentry;
//!
//! # fn main() {
//! #[launch]
//! fn rocket() -> _ {
//!     rocket::build()
//!         .attach(RocketSentry::fairing())
//!         // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^   add this line
//! }
//! # }
//! ```
//!
//! Then, the Sentry integration can be enabled by adding a `sentry_dsn=` value to
//! the `Rocket.toml` file, for example:
//!
//! ```toml
//! [debug]
//! sentry_dsn = ""  # Disabled
//! [release]
//! sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
//! sentry_transaction_sample_rate = 0.2  # 20% of requests will be logged under the performance tab
//! ```
//!
#[macro_use]
extern crate log;

use std::sync::{Arc, Mutex};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::serde::Deserialize;
use rocket::{fairing, Build, Rocket, Request, Data, Response};
use rocket::http::Status;
use rocket::request::local_cache_once;
use sentry::{ClientInitGuard, ClientOptions, protocol, Transaction};

const TRANSACTION_OPERATION_NAME: &str = "http.server";

pub struct RocketSentry {
    guard: Mutex<Option<ClientInitGuard>>,
}

#[derive(Deserialize)]
struct Config {
    sentry_dsn: String,
    sentry_transaction_sample_rate: Option<f32>,  // Default is 0 so no transaction transmitted
}

impl RocketSentry {
    pub fn fairing() -> impl Fairing {
        RocketSentry {
            guard: Mutex::new(None),
        }
    }

    fn init(&self, dsn: &str, traces_sample_rate: f32) {
        let guard = sentry::init((
            dsn,
            ClientOptions {
                before_send: Some(Arc::new(|event| {
                    info!("Sending event to Sentry: {}", event.event_id);
                    Some(event)
                })),
                traces_sample_rate,
                ..Default::default()
            },
        ));

        if guard.is_enabled() {
            // Tuck the ClientInitGuard in the fairing, so it lives as long as the server.
            let mut self_guard = self.guard.lock().unwrap();
            *self_guard = Some(guard);

            info!("Sentry enabled.");
        } else {
            error!("Sentry did not initialize.");
        }
    }

    fn start_transaction(name: &str) -> Transaction {
        let transaction_context = sentry::TransactionContext::new(
            name,
            TRANSACTION_OPERATION_NAME,
        );
        sentry::start_transaction(transaction_context)
    }

    /// Same type as the underlying function so as to retrieve a transaction from the cache.
    /// Should not be called but won't panic either.
    fn invalid_transaction() -> Transaction {
        let name = "INVALID TRANSACTION";
        Self::start_transaction(name)
    }
}

#[rocket::async_trait]
impl Fairing for RocketSentry {
    fn info(&self) -> Info {
        Info {
            name: "rocket-sentry",
            kind: Kind::Ignite | Kind::Singleton | Kind::Request | Kind::Response,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let figment = rocket.figment();

        let config: figment::error::Result<Config> = figment.extract();
        match config {
            Ok(config) => {
                if config.sentry_dsn.is_empty() {
                    info!("Sentry disabled.");
                } else {
                    let traces_sample_rate = config.sentry_transaction_sample_rate.unwrap_or(0f32);
                    self.init(&config.sentry_dsn, traces_sample_rate);
                }
            }
            Err(err) => error!("Sentry not configured: {}", err),
        }
        Ok(rocket)
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        let name = request_to_transaction_name(request);
        let build_transaction = move || Self::start_transaction(&name);
        let request_transaction = local_cache_once!(request, build_transaction);
        request.local_cache(request_transaction);
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        // We take the transaction set in the on_request callback
        let request_transaction = local_cache_once!(request, Self::invalid_transaction);
        let ongoing_transaction: &Transaction = request.local_cache(request_transaction);
        ongoing_transaction.set_status(map_status(response.status()));
        set_transaction_request(ongoing_transaction, request);
        ongoing_transaction.clone().finish();
    }
}

fn set_transaction_request(transaction: &Transaction, request: &Request) {
    transaction.set_request(protocol::Request {
        url: None,
        method: Some(String::from(request.method().as_str())),
        data: None,
        query_string: request_to_query_string(request),
        cookies: None,
        headers: Default::default(),
        env: Default::default(),
    });
}

fn request_to_transaction_name(request: &Request) -> String {
    let method = request.method();
    let path = request.uri().path();
    format!("{method} {path}")
}

fn request_to_query_string(request: &Request) -> Option<String> {
    let query_string = request.uri().query()?.as_str().to_string();
    Some(query_string)
}

fn map_status(status: Status) -> protocol::SpanStatus {
    match status.code {
        100..=299 => protocol::SpanStatus::Ok,
        300..=399 => protocol::SpanStatus::InvalidArgument,
        401 => protocol::SpanStatus::Unauthenticated,
        403 => protocol::SpanStatus::PermissionDenied,
        404 => protocol::SpanStatus::NotFound,
        409 => protocol::SpanStatus::AlreadyExists,
        429 => protocol::SpanStatus::ResourceExhausted,
        400..=499 => protocol::SpanStatus::InvalidArgument,
        501 => protocol::SpanStatus::Unimplemented,
        503 => protocol::SpanStatus::Unavailable,
        500..=599 => protocol::SpanStatus::InternalError,
        _ => protocol::SpanStatus::UnknownError,
    }
}

#[cfg(test)]
mod tests {
    use rocket::local::asynchronous::Client;
    use crate::{request_to_query_string, request_to_transaction_name};

    #[rocket::async_test]
    async fn request_to_sentry_transaction_name_get_no_path() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("/");

        let transaction_name = request_to_transaction_name(request.inner());

        assert_eq!(transaction_name, "GET /");
    }

    #[rocket::async_test]
    async fn request_to_sentry_transaction_name_get_some_path() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("/some/path");

        let transaction_name = request_to_transaction_name(request.inner());

        assert_eq!(transaction_name, "GET /some/path");
    }

    #[rocket::async_test]
    async fn request_to_sentry_transaction_name_post_path_with_variables() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.post("/users/6");

        let transaction_name = request_to_transaction_name(request.inner());

        // Ideally, we should just returns /users/<id> as configured in the routes
        assert_eq!(transaction_name, "POST /users/6");
    }

    #[rocket::async_test]
    async fn request_to_query_string_is_none() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.post("/");

        let query_string = request_to_query_string(request.inner());

        assert_eq!(query_string, None);
    }

    #[rocket::async_test]
    async fn request_to_query_string_single_parameter() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.post("/?param1=value1");

        let query_string = request_to_query_string(request.inner());

        assert_eq!(query_string, Some("param1=value1".to_string()));
    }

    #[rocket::async_test]
    async fn request_to_query_string_multiple_parameters() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.post("/?param1=value1&param2=value2");

        let query_string = request_to_query_string(request.inner());

        assert_eq!(query_string, Some("param1=value1&param2=value2".to_string()));
    }
}
