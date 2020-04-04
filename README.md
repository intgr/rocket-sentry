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

fn main() {
    rocket::ignite()
        .attach(RocketSentry::fairing())
        // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^   add this line
        .launch();
}
```

Then, the Sentry integration can be enabled by adding a `sentry_dsn=` value to
the `Rocket.toml` file, for example:

```toml
[development]
sentry_dsn = ""  # Disabled
[staging]
sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
[production]
sentry_dsn = "https://057006d7dfe5fff0fbed461cfca5f757@sentry.io/1111111"
```

Testing
-------

The functionality can be tested with the `examples/panic.rs` example. Just
change the `Rocket.toml` file and run it...

```shell script
rustup override set nightly
cargo run --example panic
```

Then try accessing this URL:
http://localhost:8012/panic?msg=Is+it+time+to+panic+yet?
