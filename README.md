# tari_log4rs
This is a fork of log4rs fixing some security bugs. This is used as a stop gap till log4rs 1.3 is released on crates.io

[![docs](https://docs.rs/log4rs/badge.svg)](https://docs.rs/log4rs)
[![crates.io](https://img.shields.io/crates/v/log4rs.svg)](https://crates.io/crates/log4rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/clippy.svg)](#license)
![CI](https://github.com/estk/log4rs/workflows/CI/badge.svg)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.56+-green.svg)](https://github.com/estk/log4rs#rust-version-requirements)

log4rs is a highly configurable logging framework modeled after Java's Logback
and log4j libraries.

### Warning

If you are using the file rotation in your configuration there is a known substantial performance issue so listen up!
By default the `gzip` feature is enabled and when rolling files it will zip log archives automatically. This is a problem
when the log archives are large as the zip happens in the main thread and will halt the process while the zip is completed.
Be advised that the `gzip` feature will be removed from default features as of `1.0`.

The methods to mitigate this are as follows.

1. Use the `background_rotation` feature which spawns an os thread to do the compression.
1. Disable the `gzip` feature with `--no-default-features`.
1. Ensure the archives are small enough that the compression time is acceptable.

For more information see the PR that added [`background_rotation`](https://github.com/estk/log4rs/pull/117).

## Quick Start

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

1.46

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
