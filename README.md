# log4rs

[![Build Status](https://travis-ci.org/sfackler/log4rs.svg?branch=master)](https://travis-ci.org/sfackler/log4rs)

Documentation is available at https://sfackler.github.io/log4rs/doc/log4rs

log4rs is a highly configurable logging framework modeled after Java's
Logback and log4j libraries.

# Architecture

The basic units of configuration are *appenders* and *loggers*.

## Appenders

An appender takes a log record and logs it somewhere, for example, to a
file, the console, or the syslog.

## Loggers

A log event is targeted at a specific logger, which are identified by
string names. The logging macros built in to the `log` crate set the logger
of a log event to the one identified by the module containing the
invocation location.

Loggers form a heirarchy: logger names are divided into components by "::".
One logger is the ancestor of another if the first logger's component list
is a prefix of the second logger's component list.

Loggers are associated with a maximum log level. Log events for that logger
with a level above the maximum will be ignored. The maximum log level for
any logger can be configured manually; if it is not, the level will be
inherited from the logger's parent.

Loggers are also associated with a set of appenders. Appenders can be
associated directly with a logger. In addition, the appenders of the
logger's parent will be associated with the logger unless the logger has
its *additivity* set to `false`. Log events sent to the logger that are not
filtered out by the logger's maximum log level will be sent to all
associated appenders.

The "root" logger is the ancestor of all other logger. Since it has no
ancestors, its additivity cannot be configured.

# Configuration

The log4rs can be configured either programmatically by using the builders
in the `config` module to construct a log4rs `Config` object, which can be
passed to the `init_config` function.

The more common configuration method, however, is via a separate TOML
config file. The `init_file` function takes the path to a config file as
well as a `Creator` object which is responsible for instantiating the
various objects specified by the config file. The `toml` module
documentation covers the exact configuration syntax, but an example is
provided below.

# Examples

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
