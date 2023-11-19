# Changelog

All notable changes to this project will be documented in this file.

The format is similar to [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.0] - 2023-02-25

- **Dependency:** Update Rust crate sentry to 0.30.0 ([#55](https://github.com/intgr/rocket-sentry/pull/55))

## [0.14.0] - 2023-01-08

- **Dependency:** Update Rust crate sentry to 0.29.1 ([#53](https://github.com/intgr/rocket-sentry/pull/53))

## [0.13.0] - 2022-11-08

- **Dependency:** Update Rust crate sentry to 0.28.0 ([#51](https://github.com/intgr/rocket-sentry/pull/51))

## [0.12.0] - 2022-09-30

- **Added** Log a message when Sentry events are sent ([#48](https://github.com/intgr/rocket-sentry/pull/48))
- **Dependency:** Update Rust crate sentry to 0.27.0 ([#44](https://github.com/intgr/rocket-sentry/pull/44))

## [0.11.0] - 2022-05-22

- **Dependency:** Update Rust crate sentry to 0.26.0 ([#42](https://github.com/intgr/rocket-sentry/pull/42))
- **Dependency:** Update Rust crate rocket to 0.5.0-rc.2 ([#39](https://github.com/intgr/rocket-sentry/pull/39))
- **Chore:** Update to Rust 2021 edition ([#41](https://github.com/intgr/rocket-sentry/pull/41))
- **CI:** Use up to date nightly Rust in CI ([#40](https://github.com/intgr/rocket-sentry/pull/40))

## [0.10.0] - 2022-03-26

- **Dependency:** Update Rust crate sentry to 0.25.0 ([#31](https://github.com/intgr/rocket-sentry/pull/31))

## [0.9.0] - 2022-01-22

- **Dependency:** Update Rust crate sentry to 0.24.1 ([#28](https://github.com/intgr/rocket-sentry/pull/28))

## [0.8.0] - 2021-11-22

- **[Breaking] Changed:** Update to Rocket 0.5-rc ([#23](https://github.com/intgr/rocket-sentry/pull/23))
  * To continue using Rocket 0.4.x, stay with rocket-sentry 0.7.0
  * Now using figment and serde for configuration (as required by Rocket)
  * The fairing no longer needs to have `Response` kind
  * RocketSentry now uses fairing kind `Singleton`

## [0.7.0] - 2021-07-13

- **Dependency:** Update sentry requirement from 0.22 to 0.23 ([#20](https://github.com/intgr/rocket-sentry/pull/20))
- **Chore:** Fix Rust 1.52 warning "panic message is not a string literal" ([#18](https://github.com/intgr/rocket-sentry/pull/18))

## [0.6.0] - 2021-01-26

- **Dependency:** Update sentry requirement from 0.20 to 0.22 ([#16](https://github.com/intgr/rocket-sentry/pull/16))

## [0.5.0] - 2020-09-15

- **Dependency:** Update sentry requirement from 0.19 to 0.20 ([#13](https://github.com/intgr/rocket-sentry/pull/13))

## [0.4.0] - 2020-08-16

- **Changed:** Use log crate instead of println ([#11](https://github.com/intgr/rocket-sentry/pull/11))

  Contributed by **Afonso Bordado**

## [0.3.0] - 2020-07-22

- **Changed:** Sentry now automatically installs panic handler, dropped from rocket-sentry ([#9](https://github.com/intgr/rocket-sentry/pull/9))
- **Dependency:** Update sentry requirement from 0.18.0 to 0.19.0 ([#9](https://github.com/intgr/rocket-sentry/pull/9))

## [0.2.0] - 2020-04-04

- **Dependency:** Update sentry requirement from 0.12.0 to 0.18.0 ([#1](https://github.com/intgr/rocket-sentry/pull/1))
- **CI:** Add CI builds and basic test case ([#3](https://github.com/intgr/rocket-sentry/pull/3), [#4](https://github.com/intgr/rocket-sentry/pull/4))
- **Documentation:** Add documentation for Rust doc ([#5](https://github.com/intgr/rocket-sentry/pull/5))

## [0.1.0] - 2019-11-25
- Initial release
