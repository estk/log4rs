# log4rs

[![Build Status](https://travis-ci.org/sfackler/log4rs.svg?branch=master)](https://travis-ci.org/sfackler/log4rs)

Documentation is available at https://sfackler.github.io/log4rs/doc/v0.3.3/log4rs

log4rs is a highly configurable logging framework modeled after Java's
Logback and log4j libraries.

log.toml:
```toml
# Scan this file for changes every 30 seconds
refresh_rate = 30

# An appender named "stdout" that writes to stdout
[appender.stdout]
kind = "console"

# An appender named "requests" that writes to a file with a custom pattern
[appender.requests]
kind = "file"
path = "log/requests.log"
pattern = "%d - %m"

# Set the default logging level to "warn" and attach the "stdout" appender to the root
[root]
level = "warn"
appenders = ["stdout"]

# Raise the maximum log level for events sent to the "app::backend::db" logger to "info"
[[logger]]
name = "app::backend::db"
level = "info"

# Route log events sent to the "app::requests" logger to the "requests" appender,
# and *not* the normal appenders installed at the root
[[logger]]
name = "app::requests"
level = "info"
appenders = ["requests"]
additive = false
```

lib.rs:
```rust
#[macro_use]
extern crate log;
extern crate log4rs;

use std::default::Default;

fn main() {
    log4rs::init_file("config/log.toml", Default::default()).unwrap();

    info!("booting up");

    ...
}
```
