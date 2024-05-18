use std::sync::Arc;
use sentry::{Hub, TransactionContext};

use rocket_sentry::RocketSentry;

/// Smoke test: check that sentry gets initialized by the fairing.
#[rocket::async_test]
async fn fairing_init() {
    let hub = Hub::main();
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
    let hub = Hub::main();
    assert!(hub.client().is_none());

    let traces_sampler = move |ctx: &TransactionContext| -> f32 {
        if matches!(ctx.name(), "GET /specific/path/1" | "GET /specific/path/2") {
            log::debug!("Dropping performance transaction");
            0.
        } else {
            log::debug!("Sending performance transaction 80% of the time");
            0.8
        }
    };
    let rocket_sentry = RocketSentry::default().set_traces_sampler(Arc::new(traces_sampler));

    let _rocket = rocket::build()
        .attach(rocket_sentry)
        .ignite()
        .await
        .expect("Rocket failed to ignite");

    assert!(hub.client().is_some());
}

