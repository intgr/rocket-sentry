#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};
use sentry::TransactionContext;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rocket_sentry::RocketSentry;

#[get("/performance")]
fn performance() -> String {
    let duration = Duration::from_millis(500);
    thread::sleep(duration);
    format!("Waited {duration:?}")
}

#[get("/performance/<id>")]
fn performance_with_id(id: u16) -> String {
    // Wait as long as the id in seconds
    let duration = Duration::from_secs(id.into());
    thread::sleep(duration);
    format!("Waited {duration:?} for id {id}")
}

#[get("/performance?<param1>&<param2>")]
fn performance_with_parameter(param1: String, param2: u32) -> String {
    let duration = Duration::from_millis(250);
    thread::sleep(duration);
    format!("Waited {duration:?} for param {param1} - {param2}")
}

#[get("/performance/skip")]
fn performance_skipped() -> String {
    let duration = Duration::from_millis(100);
    thread::sleep(duration);
    format!("Waited {duration:?}\nTransaction will be dropped")
}

#[get("/performance/random")]
fn performance_rng() -> String {
    let duration = Duration::from_millis(100);
    thread::sleep(duration);
    format!("Waited {duration:?}\nTransaction MIGHT be dropped")
}

#[launch]
fn rocket() -> Rocket<Build> {
    let rocket_instance = rocket::build();
    // Get the default configured sample rate from `Rocket.toml`
    let default_rate = rocket_instance
        .figment()
        .extract_inner::<f32>("sentry_traces_sample_rate")
        .unwrap();
    let traces_sampler = move |ctx: &TransactionContext| -> f32 {
        match ctx.name() {
            "GET /performance/skip" => {
                log::warn!("Dropping performance transaction");
                0.
            }
            "GET /performance/random" => {
                log::warn!("Sending performance transaction half the time");
                0.
            }
            _ => {
                log::warn!("Sending performance transaction using default rate");
                default_rate
            }
        }
    };
    let rocket_sentry = RocketSentry::builder()
        .traces_sampler(Arc::new(traces_sampler))
        .build();
    rocket_instance.attach(rocket_sentry).mount(
        "/",
        routes![
            performance,
            performance_with_id,
            performance_with_parameter,
            performance_skipped,
            performance_rng,
        ],
    )
}
