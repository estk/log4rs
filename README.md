# log4rs

[![docs](https://docs.rs/log4rs/badge.svg)](https://docs.rs/log4rs)
[![crates.io](https://img.shields.io/crates/v/log4rs.svg)](https://crates.io/crates/log4rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/clippy.svg)](#license)
![CI](https://github.com/estk/log4rs/workflows/CI/badge.svg)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.70+-green.svg)](https://github.com/estk/log4rs#rust-version-requirements)

log4rs is a highly configurable logging framework modeled after Java's Logback
and log4j libraries.

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

1.70

## Building for Dev

* Run the tests: `cargo test --all-features`
* Run the tests for windows with [cross](https://github.com/rust-embedded/cross):
  `cross test --target x86_64-pc-windows-gnu`
* Run the tests for all individual features: `./test.sh`
* Run the tests for all individual features for windows with
  [cross](https://github.com/rust-embedded/cross): `./test.sh win`


## Compression

If you are using the file rotation in your configuration there is a known
substantial performance issue with the `gzip` feature. When rolling files 
it will zip log archives automatically. This is a problem when the log archives
are large as the zip happens in the main thread and will halt the process while
the zip is completed.

The methods to mitigate this are as follows.

1. Use the `background_rotation` feature which spawns an os thread to do the compression.
2. Do not enable the `gzip` feature.
3. Ensure the archives are small enough that the compression time is acceptable.

For more information see the PR that added [`background_rotation`](https://github.com/estk/log4rs/pull/117).

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
