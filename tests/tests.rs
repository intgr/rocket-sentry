use rocket_sentry::RocketSentry;
use sentry::Hub;

/// Smoke test: check that sentry gets initialized by the fairing.
#[test]
fn fairing_init() {
    let hub = Hub::main();
    assert!(hub.client().is_none());

    rocket::ignite().attach(RocketSentry::fairing());
    assert!(hub.client().is_some());
}
