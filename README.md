Rocket Sentry
=============

[![Crates.io version](https://img.shields.io/crates/v/rocket-sentry.svg)](https://crates.io/crates/rocket-sentry)
[![Documentation](https://docs.rs/rocket-sentry/badge.svg)](https://docs.rs/rocket-sentry/)
[![Tests status](https://github.com/intgr/rocket-sentry/workflows/Tests/badge.svg?branch=master)](https://github.com/intgr/rocket-sentry/actions?query=workflow:Tests)

`rocket-sentry` is a simple add-on for the **Rocket** web framework to simplify
integration with the **Sentry** application monitoring system.

Or maybe...

> "The Rocket Sentry is a static rocket-firing gun platform that is based on a
> Personality Construct and used in the Aperture Science Enrichment Center."
>
> -- [Half-Life wiki](https://half-life.fandom.com/wiki/Rocket_Sentry)

Features
--------

Currently, `rocket-sentry` only enables the Rust panic handler.

`rocket-sentry` can be configured via `Rocket.toml` (`sentry_dsn=`) or
environment variable `ROCKET_SENTRY_DSN`.

Usage
-----

To use this, add the dependency to your `Cargo.toml`, and add the fairing
to your code:

```rust
use rocket_sentry::RocketSentry;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(RocketSentry::fairing())
        // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^   add this line
}
```

Then, the Sentry integration can be enabled by adding a `sentry_dsn=` value to
the `Rocket.toml` file, for example:

```toml
[debug]
sentry_dsn = ""  # Disabled
[release]
sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
```

Testing
-------

The functionality can be tested with the `examples/panic.rs` example. Just
change the `Rocket.toml` file and run it...

```shell script
cargo run --example panic
```

Then try accessing this URL:
http://localhost:8012/panic?msg=Is+it+time+to+panic+yet?

Release history
---------------
##### 0.7.0 (2021-07-13)
* Update sentry requirement from 0.22 to 0.23 (#20)

##### 0.6.0 (2021-01-26)
* Update sentry requirement from 0.20 to 0.22 (#16)

##### 0.5.0 (2020-09-15)
* Update sentry requirement from 0.19 to 0.20 (#13)

##### 0.4.0 (2020-08-16)
* Use log crate instead of println (#11).
  Thanks to Afonso Bordado, @afonso360

##### 0.3.0 (2020-07-22)
* Update sentry requirement from 0.18.0 to 0.19.0 (#9)
  * Sentry now automatically installs panic handler, dropped from rocket-sentry.

##### 0.2.0 (2020-04-04)
* Update sentry requirement from 0.12.0 to 0.18.0 (#1)
* Add CI builds and basic test case (#3, #4)
* Add documentation for Rust doc (#5)

##### 0.1.0 (2019-11-25)
* Initial release
