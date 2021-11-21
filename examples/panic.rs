#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

use rocket_sentry::RocketSentry;

#[get("/panic?<msg>")]
fn panic(msg: Option<String>) -> String {
    panic!("{}", msg.unwrap_or("You asked for it!".to_string()));
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(RocketSentry::fairing())
        .mount("/", routes![panic])
}
