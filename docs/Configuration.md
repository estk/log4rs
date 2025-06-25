# Configuration

log4rs can be configured programmatically by using the builders in the `config`
module to construct a log4rs `Config` object, which can be passed to the
`init_config` function.

The more common configuration method, however, is via a separate config file.
The `init_file` function takes the path to a config file as well as a
`Deserializers` object which is responsible for instantiating the various
objects specified by the config file. The following section covers the exact
configuration syntax. Examples of both the programmatic and configuration files
can be found in the
[examples directory](https://github.com/estk/log4rs/tree/main/examples).

## Common Fields

### LevelFilter's

- Off
- Error
- Warn
- Info
- Debug
- Trace

### Filters

The only accepted `filter` is of kind threshold with a level. The level must
be a [LevelFilter](#levelfilters). One to many filters are allowed.

i.e.

```yml
filters:
  - kind: threshold
    level: info
```

### Encoder

An `encoder` consists of a kind: the default which is pattern, or json. If
pattern is defined, the default pattern `{d} {l} {t} - {m}{n}` is used unless
overridden. Refer to
[this documentation](https://docs.rs/log4rs/latest/log4rs/encode/pattern/index.html#formatters)
for details regarding valid patterns.

> Note that the json encoder does not have any additional controls such as the
> pattern field.

i.e.

```yml
encoder:
  kind: pattern
  pattern: "{h({d(%+)(utc)} [{f}:{L}] {l:<6} {M}:{m})}{n}"
```

## Loggers

A map of logger configurations.

### Logger Configuration

The _name_ of the logger is the yml tag.

The _level_ of the logger is optional and defaults to the parents log level.
The level must be a [LevelFilter](#levelfilters).

The _appenders_ field is an optional list of [appenders](#appenders) attached
to the logger.

The _additive_ field is an optional boolean determining if the loggers parent
will also be attached to this logger. The default is true.

i.e.

```yml
loggers:
  my_logger:
    level: info
    appenders:
      - my_appender
    additive: true
```

## The Root Logger

Root is the required logger. It is the parent to all children loggers. To
configure the Root, refer to [the logger section](#logger-configuration).

> Note: The root logger has no parent, and therefore the _additive_
field does not apply.

```yml
root:
  level: info
  appenders:
    - my_appender
```

## Appenders

All appenders require a unique identifying string for each
[appender configuration](#appender-config).

### Appender Config

Each Appender Kind has it's own configuration. However, all accept
[filters](#filters). The `kind` field is required in an appender configuration.

#### The Console Appender

The _target_ field is optional and accepts `stdout` or `stderr`. It's default
value is stdout.

The _tty_only_ field is an optional boolean and dictates that the appender must
only write when the target is a TTY. It's default value is false.

The _encoder_ field is optional and can consist of multiple fields. Refer to
the [encoder](#encoder) documention.

```yml
my_console_appender:
  kind: console
  target: stdout
  tty_only: false
```

#### The File Appender

The _path_ field is required and accepts environment variables of the form
`$ENV{name_here}`. The path can be relative or absolute.

The _path_ field also supports date/time formats such as `$TIME{chrono_format}`. Refer
to [chrono format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for date and time formatting syntax

**Note:** There is a maximum of 5 `$TIME{...}` replacements per path. If more than 5 `$TIME{...}` placeholders are present, only the first 5 will be replaced; the rest will remain unchanged in the path.

The _encoder_ field is optional and can consist of multiple fields. Refer to
the [encoder](#encoder) documention.

The _append_ field is an optional boolean and defaults to `true`. True will
append to the log file if it exists, false will truncate the existing file.

```yml
my_file_appender:
  kind: file
  path: $ENV{PWD}/log/test_$TIME{%Y-%m-%d_%H-%M-%S}.log
  append: true
```

#### The Rolling File Appender

The rolling file configuration is by far the most complex. Like the
[file appender](#the-file-appender), the path to the log file is required
with the _append_ and the _encoders_ optional fields.

i.e.

```yml
my_rolling_appender:
  kind: rolling_file
  path: "logs/test.log"
  policy:
    kind: compound
    trigger:
      kind: size
      limit: 1mb
    roller:
      kind: fixed_window
      base: 1
      count: 5
      pattern: "logs/test.{}.log"
```

The new component is the _policy_ field. A policy must have the _kind_ field like most
other components, the default (and only supported) policy is `kind: compound`.

The _trigger_ field is used to dictate when the log file should be rolled. It
supports three types: `size`, `time` and `onstartup`.

For `size`, it require a _limit_ field. The _limit_ field is a string which defines the maximum file size
prior to a rolling of the file. The limit field requires one of the following
units in bytes, case does not matter:

- b
- kb/kib
- mb/mib
- gb/gib
- tb/tib

i.e.

```yml
trigger:
  kind: size
  limit: 10 mb
```

For `time`, it has three field, _interval_, _modulate_ and _max_random_delay_.

The _interval_ field is a string which defines the time to roll the
file. The interval field supports the following units(second will be used if the
unit is not specified), case does not matter:

- second[s]
- minute[s]
- hour[s]
- day[s]
- week[s]
- month[s]
- year[s]

> Note: `log4j` treats `Sunday` as the first day of the week, but `log4rs` treats
> `Monday` as the first day of the week, which follows the `chrono` crate
> and the `ISO 8601` standard. So when using `week`, the log file will be rolled
> on `Monday` instead of `Sunday`.

The _modulate_ field is an optional boolean. It indicates whether the interval should
be adjusted to cause the next rollover to occur on the interval boundary. For example,
if the interval is 4 hours and the current hour is 3 am, when true, the first rollover
will occur at 4 am and then next ones will occur at 8 am, noon, 4pm, etc. The default
value is false.

The _max_random_delay_ field is an optional integer. It indicates the maximum number
of seconds to randomly delay a rollover. By default, this is 0 which indicates no
delay. This setting is useful on servers where multiple applications are configured
to rollover log files at the same time and can spread the load of doing so across
time.

i.e.

```yml
trigger:
    kind: time
    interval: 1 day
    modulate: false
    max_random_delay: 0
```

For `onstartup`, it has an optional field, _min_size_. It indicates the minimum size the file must have to roll over. A size of zero will cause a roll over no matter what the file size is. The default value is 1, which will prevent rolling over an empty file.

i.e.

```yml
trigger:
    kind: onstartup
    min_size: 1
```

The _roller_ field supports two types: delete, and fixed_window. The delete
roller does not take any other configuration fields. The fixed_window roller
supports three fields: pattern, base, and count. The most current log file will
always have the _base_ index.

The _pattern_ field is used to rename files. The pattern must contain the
double curly brace `{}`. For example `archive/foo.{}.log`. Each instance of
`{}` will be replaced with the index number of the configuration file. Note
that if the file extension of the pattern is `.gz` and the `gzip` Cargo
feature is enabled, the archive files will be gzip-compressed.
If the file extension of the pattern is `.zst` and the `zstd` Cargo
feature is enabled, the archive files will be compressed using the 
[Zstandard](https://facebook.github.io/zstd/) compression algorithm.

> Note: This pattern field is only used for archived files. The `path` field
> of the higher level `rolling_file` will be used for the active log file.

The _base_ field is the starting index used to name rolling files.

The _count_ field is the exclusive maximum index used to name rolling files.
However, be warned that the roller renames every file when a log rolls over.
Having a large count value can negatively impact performance.

> Note: If you use the `triger: time`, the log file will be rolled before it
> gets written, which ensures that the logs are rolled in the correct position
> instead of leaving a single line of logs in the previous log file. However,
> this may cause a substantial slowdown if the `background` feature is not enabled.

i.e.

```yml
roller:
  kind: fixed_window
  base: 1
  count: 5
  pattern: "archive/journey-service.{}.log"
```

or

```yml
roller:
  kind: delete
```

## Refresh Rate

The _refresh_rate_ accepts a u64 value in seconds. The field is used to
determine how often log4rs will scan the configuration file for changes. If a
change is discovered, the logger will reconfigure automatically.

i.e.

```yml
refresh_rate: 30 seconds
```
