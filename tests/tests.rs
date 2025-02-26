use figment::Figment;
use rocket::Config;
use rocket_sentry::RocketSentry;
use sentry::{Hub, TransactionContext};
use std::sync::Arc;

const SENTRY_DSN_CONFIG: (&str, &str) = ("sentry_dsn", "https://123@sentry.io/456");

/// Smoke test: check that sentry gets initialized by the fairing.
#[rocket::async_test]
async fn fairing_init() {
    let hub = Hub::current();
    assert!(hub.client().is_none());

    let _rocket = rocket::build()
        .attach(RocketSentry::fairing())
        .ignite()
        .await
        .expect("Rocket failed to ignite");

    assert!(hub.client().is_some());
}

#[rocket::async_test]
async fn fairing_init_with_specific_traces_sampler() {
    let hub = Hub::current();
    assert!(hub.client().is_none());

    let traces_sampler = move |ctx: &TransactionContext| -> f32 {
        match ctx.name() {
            "GET /specific/path/1" | "GET /specific/path/2" => {
                log::debug!("Dropping performance transaction");
                0.
            }
            _ => {
                log::debug!("Sending performance transaction 80% of the time");
                0.8
            }
        }
    };
    let rocket_sentry = RocketSentry::builder()
        .traces_sampler(Arc::new(traces_sampler))
        .build();

    let _rocket = rocket::build()
        .attach(rocket_sentry)
        .ignite()
        .await
        .expect("Rocket failed to ignite");

    assert!(hub.client().is_some());
}

async fn init_rocket_using_figment(figment: Figment) {
    rocket::custom(figment)
        .attach(RocketSentry::fairing())
        .ignite()
        .await
        .expect("Rocket failed to ignite");
}

fn sentry_current_hub_environment() -> String {
    let client = Hub::current().client().unwrap();
    let client_options = client.options();
    client_options.environment.clone().unwrap().to_string()
}

#[rocket::async_test]
async fn fairing_init_with_debug_rocket_profile() {
    let figment = Figment::from(Config::debug_default()).join(SENTRY_DSN_CONFIG);
    init_rocket_using_figment(figment).await;

    assert_eq!(sentry_current_hub_environment(), "development"); // default to development for debug build
}

#[rocket::async_test]
async fn fairing_init_with_release_rocket_profile() {
    let figment = Figment::from(Config::release_default()).join(SENTRY_DSN_CONFIG);
    init_rocket_using_figment(figment).await;

    assert_eq!(sentry_current_hub_environment(), "production"); // default to production for release build
}

#[rocket::async_test]
async fn fairing_init_with_custom_rocket_profile() {
    let profile_name = "staging";
    let figment = Figment::new()
        .select(profile_name)
        .join(Config::debug_default())
        .join(SENTRY_DSN_CONFIG);

    init_rocket_using_figment(figment).await;

    assert_eq!(sentry_current_hub_environment(), profile_name); // Rocket profile name was passed to Sentry config
}
