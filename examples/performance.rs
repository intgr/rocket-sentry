#[macro_use]
extern crate rocket;

use std::thread;
use std::time::Duration;
use rocket::{Build, Rocket};

use rocket_sentry::RocketSentry;

#[get("/performance")]
fn performance() -> String {
    let duration = Duration::from_millis(500);
    thread::sleep(duration);
    return format!("Waited {duration:?}");
}

#[get("/performance/<id>")]
fn performance_with_id(id: u16) -> String {
    // Wait as long as the id in secondes
    let duration = Duration::from_secs(id.into());
    thread::sleep(duration);
    return format!("Waited {duration:?} for id {id}");
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(RocketSentry::fairing())
        .mount("/", routes![performance, performance_with_id])
}
