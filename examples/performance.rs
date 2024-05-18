#[macro_use]
extern crate rocket;

use std::sync::Arc;
use rocket::{Build, Rocket};
use std::thread;
use std::time::Duration;
use sentry::TransactionContext;

use rocket_sentry::RocketSentry;

#[get("/performance")]
fn performance() -> String {
    let duration = Duration::from_millis(500);
    thread::sleep(duration);
    return format!("Waited {duration:?}");
}

#[get("/performance/<id>")]
fn performance_with_id(id: u16) -> String {
    // Wait as long as the id in seconds
    let duration = Duration::from_secs(id.into());
    thread::sleep(duration);
    return format!("Waited {duration:?} for id {id}");
}

#[get("/performance?<param1>&<param2>")]
fn performance_with_parameter(param1: String, param2: u32) -> String {
    let duration = Duration::from_millis(250);
    thread::sleep(duration);
    return format!("Waited {duration:?} for param {param1} - {param2}");
}

#[get("/performance/skip")]
fn performance_skipped() -> String {
    let duration = Duration::from_millis(100);
    thread::sleep(duration);
    return format!("Waited {duration:?}\nTransaction will be dropped");
}

#[get("/performance/rng")]
fn performance_rng() -> String {
    let duration = Duration::from_millis(100);
    thread::sleep(duration);
    return format!("Waited {duration:?}\nTransaction MIGHT be dropped");
}

#[launch]
fn rocket() -> Rocket<Build> {
    let traces_sampler = move |ctx: &TransactionContext| -> f32 {
        if ctx.name().to_lowercase().contains("skip") {
            log::warn!("Dropping performance transaction");
            0.
        } else if ctx.name().to_lowercase().contains("rng") {
            log::warn!("Sending performance transaction half the time");
            0.5
        } else {
            log::warn!("Sending performance transaction");
            1.
        }
    };
    let rocket_sentry = RocketSentry::new().set_traces_sampler(Arc::new(traces_sampler));
    rocket::build().attach(rocket_sentry).mount(
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
