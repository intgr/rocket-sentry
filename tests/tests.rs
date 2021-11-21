use sentry::Hub;

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
