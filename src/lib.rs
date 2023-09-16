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
//! ```
//!
#[macro_use]
extern crate log;

use std::sync::{Arc, Mutex};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::serde::Deserialize;
use rocket::{fairing, Build, Rocket, Request, Data, Response};
use rocket::request::local_cache_once;
use sentry::{ClientInitGuard, ClientOptions, Transaction};

const TRANSACTION_OPERATION_NAME: &str = "http.server";

pub struct RocketSentry {
    guard: Mutex<Option<ClientInitGuard>>,
}

#[derive(Deserialize)]
struct Config {
    sentry_dsn: String,
}

impl RocketSentry {
    pub fn fairing() -> impl Fairing {
        RocketSentry {
            guard: Mutex::new(None),
        }
    }

    fn init(&self, dsn: &str) {
        let guard = sentry::init((
            dsn,
            ClientOptions {
                before_send: Some(Arc::new(|event| {
                    info!("Sending event to Sentry: {}", event.event_id);
                    Some(event)
                })),
                traces_sample_rate: 1.0,  // TODO set it via config and default to 0
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

    fn build_transaction(name: &str) -> Transaction {
        let transaction_context = sentry::TransactionContext::new(
            name,
            TRANSACTION_OPERATION_NAME,
        );
        sentry::start_transaction(transaction_context)
    }

    fn invalid_transaction() -> Transaction {
        let name = "INVALID TRANSACTION";
        Self::build_transaction(name)
    }
}

pub fn request_to_transaction_name(request: &Request) -> String {
    let method = request.method();
    let path = request.uri().path();
    format!("{method} {path}")
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
                    self.init(&config.sentry_dsn);
                }
            }
            Err(err) => error!("Sentry not configured: {}", err),
        }
        Ok(rocket)
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        let name = request_to_transaction_name(request);
        let build_transaction = move || Self::build_transaction(&name);
        let request_transaction = local_cache_once!(request, build_transaction);
        request.local_cache(request_transaction);
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        // We take the transaction set in the on_request callback
        let request_transaction = local_cache_once!(request, Self::invalid_transaction);
        let ongoing_transaction: &Transaction = request.local_cache(request_transaction);
        // TODO ongoing_transaction.set_status(response.status());
        ongoing_transaction.clone().finish();
    }
}
