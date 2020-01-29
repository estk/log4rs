# log4rs

[![docs](https://docs.rs/log4rs/badge.svg)](https://docs.rs/log4rs)
[![crates.io](https://img.shields.io/crates/v/log4rs.svg)](https://crates.io/crates/log4rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/clippy.svg)](#license)
![CI](https://github.com/estk/log4rs/workflows/CI/badge.svg)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.38+-green.svg)](https://github.com/estk/log4rs#rust-version-requirements)

log4rs is a highly configurable logging framework modeled after Java's Logback
and log4j libraries.

[Documentation](https://docs.rs/log4rs)

log4rs.yaml:
```yaml
refresh_rate: 30 seconds
appenders:
  stdout:
    kind: console
  requests:
    kind: file
    path: "log/requests.log"
    encoder:
      pattern: "{d} - {m}{n}"
root:
  level: warn
  appenders:
    - stdout
loggers:
  app::backend::db:
    level: info
  app::requests:
    level: info
    appenders:
      - requests
    additive: false
```

lib.rs:
```rust
use log::{error, info, warn};
use log4rs;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("booting up");

    // ...
}
```

## Rust Version Requirements

Since version `0.10.0` the minimum version for rust is `1.38.0` and is verified via CI.
`1.38.0` is required for windows due to the [backtrace](https://github.com/rust-lang/backtrace-rs/issues/276) crate.
`1.34.0` may work for unix environments although it is not officially supported.

## Building for Dev

* Run the tests: `cargo test --all-features`
* Run the tests for windows with [cross](https://github.com/rust-embedded/cross): `cross test --target x86_64-pc-windows-gn`
* Run the tests for all individual features: `./test.sh`
* Run the tests for all individual features for windows with [cross](https://github.com/rust-embedded/cross): `./test.sh win`

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
