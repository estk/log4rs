# Configuration Explained

An in depth breakdown of the configuration options available with the Log4rs logger.

## Common Fields

### LevelFilter's:
- Off
- Error
- Warn
- Info
- Debug
- Trace

### filters
The only accepted Filter is of kind threshold with a level. The level must be a [LevelFilter](#levelfilters). One to many filters are allowed.

i.e.
```yml
filters:
   - kind: threshold
     level: info
```

### Encoder
An encoder consists of a kind: the default pattern, or json. If pattern is defined, the default pattern `{d} {l} {t} - {m}{n}` is used unless overridden. Refer to [this documentation](https://docs.rs/log4rs/0.8.3/log4rs/encode/pattern/index.html#formatters) for details regarding valid patterns. 

> Note that the json encoder does not have any additional controls such as the pattern field.

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

The _level_ of the logger is optional and defaults to the parents log level. The level must be a [LevelFilter](#levelfilters).

The _appenders_ field is an optional list of [appenders](#appenders) attached to the logger.

The _additive_ field is an optional boolean determining if the loggers parent will also be attached to this logger. The default is true.

i.e.
```yml
loggers:
   my_logger:
      level: info
      appenders:
         - my_appender
      additive: true
```

## Root

Root is the required logger. It is the parent to all children loggers. To configure the Root, refer to [the logger section](#logger-configuration).

> Note that the root logger has no parent and therefore cannot use the _additive_ field.

```yml
root:
  level: info
  appenders:
    - my_appender
```

## Appenders
All appenders require a unique identifying string for each [appender configuration](#appender-config).

### Appender Config
Each Appender Kind has it's own configuration. However, all accept [filters](#filters). The `kind` field is required in an appender configuration.

#### The console appender
The _target_ field is optional and accepts `stdout` or `stderr`. It's default value is stdout. 

The _tty_only_ field is an optional boolean and dictates that the appender must only write when the target is a TTY. It's default value is false.

The _encoder_ field is optional and can consist of multiple fields. Refer to the [encoder](#encoder) documention.

```yml
my_console_appender:
   kind: console
   target: stdout
   tty_only: false
```

#### The file appender
The _path_ field is required and accepts environment variables of the form `$ENV{name_here}`. The path can be relative or absolute.

The _encoder_ field is optional and can consist of multiple fields. Refer to the [encoder](#encoder) documention.

The _append_ field is an optional boolean and defaults to `true`. True will append to the log file if it exists, false will truncate the existing file.

```yml
my_file_appender:
   kind: file
   path: $ENV{PWD}/log/test.log
   append: true
```

#### The rolling_file appender
The rolling file configuration is by far the most complex. Like the [file appender](#the-file-appender), the path to the log file is required with the _append_ and the _encoders_ optional fields.

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

The new component is the _policy_ field. A policy must have `kind` like most other components, the default (and only supported) policy is `kind: compound`.

The _trigger_ field is used to dictate when the log file should be rolled. The only supported trigger is  `kind: size`. There is a required field `limit` which defines the maximum file size prior to a rolling of the file. The limit field requires one of the following units in bytes, case does not matter:
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

The _roller_ field supports two types: delete, and fixed_window. The delete roller does not take any other configuration fields. The fixed_window roller supports three fields: pattern, base, and count. The most current log file will always have the _base_ index.

The _pattern_ field is used to rename files. The pattern must contain the double curly brace `{}`. For example `archive/foo.{}.log`. Each instance of `{}` will be replaced with the index number of the configuration file. Note that if the file extension of the pattern is `.gz` and the `gzip` Cargo feature is enabled, the archive files will be gzip-compressed. 

> Note that this pattern field is only used for archived files. The `path` field of the higher level `rolling_file` will be used for the active log file.

The _base_ field is the starting index used to name rolling files.

The _count_ field is the exclusive maximum index used to name rolling files. However, be warned that the roller renames every file when a log rolls over. Having a large count value can negatively impact performance.

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
The _refresh_rate_ accepts a u64 value in seconds. The field is used to determine how often log4rs will scan the configuration file for changes. If a change is discovered, the logger will reconfigure automatically.

i.e.
```yml
refresh_rate: 30 seconds
```
