#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_sentry::RocketSentry;

#[get("/panic?<msg>")]
fn panic(msg: Option<String>) -> String {
    let msg = msg.unwrap_or("You asked for it!".to_string());
    panic!(msg);
}

fn main() {
    rocket::ignite()
        .attach(RocketSentry::fairing())
        .mount("/", routes![panic])
        .launch();
}
