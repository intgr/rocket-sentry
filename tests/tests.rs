use rocket::local::asynchronous::Client;
use sentry::Hub;

use rocket_sentry::{request_to_transaction_name, RocketSentry};

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