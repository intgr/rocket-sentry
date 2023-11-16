#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};
use std::thread;
use std::time::Duration;

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

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().attach(RocketSentry::fairing()).mount(
        "/",
        routes![performance, performance_with_id, performance_with_parameter],
    )
}
