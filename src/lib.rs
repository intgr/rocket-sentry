#![allow(clippy::needless_doctest_main)]
//! **rocket-sentry** is a simple add-on for the **Rocket** web framework to simplify
//! integration with the **Sentry** application monitoring system.
//!
//! Or maybe...
//!
//! > "The Rocket Sentry is a static rocket-firing gun platform that is based on a
//! > Personality Construct and used in the Aperture Science Enrichment Center."
//! >
//! > -- [Half-Life wiki](https://half-life.fandom.com/wiki/Rocket_Sentry)
//!
//! Example usage
//! =============
//!
//! ```no_run
//! use rocket_sentry::RocketSentry;
//!
//! fn main() {
//!     rocket::ignite()
//!         .attach(RocketSentry::fairing())
//!         // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^   add this line
//!         .launch();
//! }
//! ```
//!
//! Then, the Sentry integration can be enabled by adding a `sentry_dsn=` value to
//! the `Rocket.toml` file, for example:
//!
//! ```toml
//! [development]
//! sentry_dsn = ""  # Disabled
//! [staging]
//! sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
//! [production]
//! sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
//! ```
//!
use std::sync::Mutex;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::Rocket;
use sentry::internals::ClientInitGuard;

pub struct RocketSentry {
    guard: Mutex<Option<ClientInitGuard>>,
}

impl RocketSentry {
    pub fn fairing() -> impl Fairing {
        RocketSentry {
            guard: Mutex::new(None),
        }
    }

    fn init(&self, dsn: &str) {
        let guard = sentry::init(dsn);

        if guard.is_enabled() {
            // Tuck the ClientInitGuard in the fairing, so it lives as long as the server.
            let mut self_guard = self.guard.lock().unwrap();
            *self_guard = Some(guard);

            self.configure();
            println!("Sentry enabled.");
        } else {
            println!("Sentry did not initialize.");
        }
    }

    fn configure(&self) {
        sentry::integrations::panic::register_panic_handler();
    }
}

impl Fairing for RocketSentry {
    fn info(&self) -> Info {
        Info {
            name: "rocket-sentry",
            // Kind::Response is necessary, otherwise Rocket dealloc's our SentryGuard reference.
            kind: Kind::Attach | Kind::Response,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        match rocket.config().get_str("sentry_dsn") {
            Ok("") => {
                println!("Sentry disabled.");
            }
            Ok(dsn) => {
                self.init(dsn);
            }
            Err(err) => println!("Sentry disabled: {}", err),
        }
        Ok(rocket)
    }
}
