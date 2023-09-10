#[macro_use]
extern crate rocket;

use std::thread;
use std::time::Duration;
use rocket::{Build, Rocket};

use rocket_sentry::RocketSentry;

#[get("/performance")]
fn panic() -> String {
    let duration = Duration::from_millis(500);
    thread::sleep(duration);
    return format!("Waited {duration:?}");
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(RocketSentry::fairing())
        .mount("/", routes![panic])
}
