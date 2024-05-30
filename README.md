Rocket Sentry
=============

[![Crates.io version](https://img.shields.io/crates/v/rocket-sentry.svg)](https://crates.io/crates/rocket-sentry)
[![Documentation](https://docs.rs/rocket-sentry/badge.svg)](https://docs.rs/rocket-sentry/)
[![Tests status](https://github.com/intgr/rocket-sentry/workflows/Tests/badge.svg?branch=master)](https://github.com/intgr/rocket-sentry/actions?query=workflow:Tests)
[![Changelog](https://img.shields.io/badge/Changelog-f15d30.svg)](https://github.com/intgr/rocket-sentry/blob/master/CHANGELOG.md)

`rocket-sentry` is a simple add-on for the **Rocket** web framework to simplify
integration with the **Sentry** application monitoring system.

Or maybe...

> "The Rocket Sentry is a static rocket-firing gun platform that is based on a
> Personality Construct and used in the Aperture Science Enrichment Center."
>
> -- [Half-Life wiki](https://half-life.fandom.com/wiki/Rocket_Sentry)

Features
--------

Currently `rocket-sentry` includes two integrations:

* **Rust panic handler:** when a panic happens, it is reported as a Sentry event.
* **Performance Monitoring:** HTTP requests are reported as [Transactions](https://docs.sentry.io/product/performance/transaction-summary/),
  if the `sentry_traces_sample_rate` setting is configured or `traces_sampler` callback is provided (see example below).

  Transactions currently include the following fields:
  - [X] HTTP method
  - [X] GET query string
  - [X] headers
  - [ ] POST data
  - [ ] cookies
  - [ ] environment
  - [ ] URL

Pull requests welcome!

Usage
-----

`rocket-sentry` can be configured via `Rocket.toml` (`sentry_dsn=`) or
environment variable `ROCKET_SENTRY_DSN`.

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
sentry_traces_sample_rate = 0.2  # 20% of requests will be logged under the performance tab
```

`traces_sampler` can be used instead of `sentry_traces_sample_rate` to have a more granular control, [see details here](https://docs.sentry.io/platforms/rust/configuration/sampling/#configuring-the-transaction-sample-rate).
```rust
use rocket_sentry::RocketSentry;

#[launch]
fn rocket() -> _ {
    let traces_sampler = move |ctx: &TransactionContext| -> f32 {
        match ctx.name() {
            path if matches!(path, "GET /specific/path/1" | "GET /specific/path/2") => 0.,  // Drop the performance transaction
            _ => 1.,
        }
    };
    rocket::build()
        .attach(RocketSentry::builder().traces_sampler(Arc::new(traces_sampler)).build());
}
```
See [a more advanced example](examples/performance.rs).

Testing
-------

The functionality can be tested with the `examples/panic.rs` example. Just
change the `Rocket.toml` file and run it...

```shell script
cargo run --example panic
```

Then try accessing this URL:
http://localhost:8012/panic?msg=Is+it+time+to+panic+yet?
