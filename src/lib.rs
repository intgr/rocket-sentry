#![warn(clippy::pedantic)]
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
//! sentry_traces_sample_rate = 0.2  # 20% of requests will be logged under the performance tab
//! ```
//!
#[macro_use]
extern crate log;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::local_cache_once;
use rocket::serde::Deserialize;
use rocket::{fairing, Build, Data, Request, Response, Rocket};
use sentry::protocol::SpanStatus;
use sentry::{protocol, ClientInitGuard, ClientOptions, TracesSampler, Transaction};

const TRANSACTION_OPERATION_NAME: &str = "http.server";

pub struct RocketSentry {
    guard: Mutex<Option<ClientInitGuard>>,
    transactions_enabled: AtomicBool,
    traces_sampler: Option<Arc<TracesSampler>>,
}

#[derive(Deserialize)]
struct Config {
    sentry_dsn: String,
    sentry_traces_sample_rate: Option<f32>, // Default is 0 so no transaction transmitted
}

impl RocketSentry {
    #[must_use]
    pub fn fairing() -> impl Fairing {
        RocketSentry::builder().build()
    }

    #[must_use]
    pub fn builder() -> RocketSentryBuilder {
        RocketSentryBuilder::default()
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
                traces_sampler: self.traces_sampler.clone(),
                ..Default::default()
            },
        ));

        if guard.is_enabled() {
            // Tuck the ClientInitGuard in the fairing, so it lives as long as the server.
            let mut self_guard = self.guard.lock().unwrap();
            *self_guard = Some(guard);

            info!("Sentry enabled.");
            if traces_sample_rate > 0f32 || self.traces_sampler.is_some() {
                self.transactions_enabled.store(true, Ordering::Relaxed);
            }
        } else {
            error!("Sentry did not initialize.");
        }
    }

    fn start_transaction(name: &str) -> Transaction {
        let transaction_context = sentry::TransactionContext::new(name, TRANSACTION_OPERATION_NAME);
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
                    let traces_sample_rate = config.sentry_traces_sample_rate.unwrap_or(0f32);
                    self.init(&config.sentry_dsn, traces_sample_rate);
                }
            }
            Err(err) => error!("Sentry not configured: {}", err),
        }
        Ok(rocket)
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        if self.transactions_enabled.load(Ordering::Relaxed) {
            let name = request_to_transaction_name(request);
            let build_transaction = move || Self::start_transaction(&name);
            let request_transaction = local_cache_once!(request, build_transaction);
            request.local_cache(request_transaction);
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if self.transactions_enabled.load(Ordering::Relaxed) {
            // We take the transaction set in the on_request callback
            let request_transaction = local_cache_once!(request, Self::invalid_transaction);
            let ongoing_transaction: &Transaction = request.local_cache(request_transaction);
            ongoing_transaction.set_status(map_status(response.status()));
            set_transaction_request(ongoing_transaction, request);
            ongoing_transaction.clone().finish();
        }
    }
}

fn set_transaction_request(transaction: &Transaction, request: &Request) {
    transaction.set_request(protocol::Request {
        url: None,
        method: Some(request.method().to_string()),
        data: None,
        query_string: request_to_query_string(request),
        cookies: None,
        headers: request_to_header_map(request),
        env: BTreeMap::new(),
    });
}

fn request_to_transaction_name(request: &Request) -> String {
    let method = request.method();
    let path = request.uri().path();
    format!("{method} {path}")
}

fn request_to_query_string(request: &Request) -> Option<String> {
    Some(request.uri().query()?.to_string())
}

fn map_status(status: Status) -> SpanStatus {
    #[allow(clippy::match_same_arms)]
    match status.code {
        100..=299 => SpanStatus::Ok,
        // For 3xx there is no appropriate redirect status, so we default to Ok as flask does,
        // https://github.com/getsentry/sentry-python/blob/e0d7bb733b5db43531b1efae431669bfe9e63908/sentry_sdk/tracing.py#L408-L435
        300..=399 => SpanStatus::Ok,
        401 => SpanStatus::Unauthenticated,
        403 => SpanStatus::PermissionDenied,
        404 => SpanStatus::NotFound,
        409 => SpanStatus::AlreadyExists,
        429 => SpanStatus::ResourceExhausted,
        400..=499 => SpanStatus::InvalidArgument,
        501 => SpanStatus::Unimplemented,
        503 => SpanStatus::Unavailable,
        500..=599 => SpanStatus::InternalError,
        _ => SpanStatus::UnknownError,
    }
}

fn request_to_header_map(request: &Request) -> BTreeMap<String, String> {
    request
        .headers()
        .iter()
        .map(|header| (header.name().to_string(), header.value().to_string()))
        .collect()
}

#[derive(Default)]
pub struct RocketSentryBuilder {
    traces_sampler: Option<Arc<TracesSampler>>,
}

impl RocketSentryBuilder {
    #[must_use]
    pub fn new() -> RocketSentryBuilder {
        RocketSentryBuilder {
            traces_sampler: None,
        }
    }

    #[must_use]
    pub fn traces_sampler(mut self, traces_sampler: Arc<TracesSampler>) -> RocketSentryBuilder {
        self.traces_sampler = Some(traces_sampler);
        self
    }

    #[must_use]
    pub fn build(self) -> RocketSentry {
        RocketSentry {
            guard: Mutex::new(None),
            transactions_enabled: AtomicBool::new(false),
            traces_sampler: self.traces_sampler,
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::ContentType;
    use rocket::http::Header;
    use rocket::local::asynchronous::Client;
    use sentry::TransactionContext;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    use crate::{
        request_to_header_map, request_to_query_string, request_to_transaction_name, RocketSentry,
    };

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

        assert_eq!(
            query_string,
            Some("param1=value1&param2=value2".to_string())
        );
    }

    #[rocket::async_test]
    async fn request_to_header_map_is_empty() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client.get("/");

        let header_map = request_to_header_map(request.inner());

        assert!(header_map.is_empty());
    }

    #[rocket::async_test]
    async fn request_to_header_map_multiple() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).await.unwrap();
        let request = client
            .get("/")
            .header(ContentType::JSON)
            .header(Header::new("custom-key", "custom-value"));

        let header_map = request_to_header_map(request.inner());

        assert_eq!(
            header_map.get("custom-key"),
            Some(&"custom-value".to_string())
        );
        assert_eq!(
            header_map.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    /// Transaction are only enabled on either a positive traces_sample_rate or a set traces_sampler
    #[rocket::async_test]
    async fn transactions_not_enabled() {
        let rocket_sentry = RocketSentry::builder().build();

        rocket_sentry.init("https://user@some.dsn/123", 0.);

        assert_eq!(
            rocket_sentry.transactions_enabled.load(Ordering::Relaxed),
            false
        );
    }

    #[rocket::async_test]
    async fn transactions_enabled_by_traces_sample_rate() {
        let rocket_sentry = RocketSentry::builder().build();

        rocket_sentry.init("https://user@some.dsn/123", 0.01);

        assert_eq!(
            rocket_sentry.transactions_enabled.load(Ordering::Relaxed),
            true
        );
    }

    #[rocket::async_test]
    async fn transactions_enabled_by_traces_sampler() {
        let rocket_sentry = RocketSentry::builder()
            .traces_sampler(Arc::new(move |_: &TransactionContext| -> f32 {
                0. // Even a sampler that deny all transaction will mark transactions as enabled
            }))
            .build();

        rocket_sentry.init("https://user@some.dsn/123", 0.);

        assert_eq!(
            rocket_sentry.transactions_enabled.load(Ordering::Relaxed),
            true
        );
    }
}
