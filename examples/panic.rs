#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_sentry::RocketSentry;

#[get("/panic?<msg>")]
fn panic(msg: Option<String>) -> String {
    panic!("{}", msg.unwrap_or("You asked for it!".to_string()));
}

fn main() {
    rocket::ignite()
        .attach(RocketSentry::fairing())
        .mount("/", routes![panic])
        .launch();
}
