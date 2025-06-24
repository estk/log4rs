# Change Log

## [1.4.0-rc2]

### Changed

* update mock_instant and small refactor (#424 @CosminPerRam)
* remove oncecell dependency (#423 @CosminPerRam)

## [1.4.0-rc1]

### New

* Support for Key-Value pairs (#362 @ellttBen)
* Add time serialization into log file (#374 @TuEmb)
* Public `TimeTriggerConfig` fields (#370 @Dirreke)
* Left truncation unicode support (#285 @moh-eulith)
* Zstd compression for log files (#363 @cristian-prato)
* Add onstartup trigger (#343 @Dirreke)
* Add config parsing tests (#357 @bconn98)
* Add handle retrieval after log initialization (#393 @izolyomi)

### Changed

* MSRV to 1.75
* Update deps: (thread-id, thiserror, mock_instant, rand)
* Remove derivative crate (#408 @royb3)
* Remove where_clauses_object_safety lint allow (#377 @Dirreke)
* Refactor of time trigger logic (#347 @Dirreke)
* Readme updated (#361 @bconn98)

## [1.3.0]

### New

* Add debug and release formatters
* Documentation on configuring the tool
* Code Coverage CI
* CVE Audit CI
* EditorConfig CI
* Code Owners
* NO_COLOR, CLICOLOR, CLICOLOR_FORCE controls
* Example of inline configuration with file rotation
* Time Based Trigger

### Changed

* Update minimum supported rust to 1.69 for CVE-2020-26235
* Update `arc-swap` to `1.6`
* Update `log` to `0.4.20`
* Update `humantime` to `2.1`
* Update `serde_yaml` to `0.9`
* Update `toml` to `0.8`
* Update `derivative` to `2.2`
* Update `tempfile` to `3.8`
* Moved `level` field before `message` in json format
* Legacy test moved to examples


### Fixed

* README typo regarding building for dev on windows
* Apply editorconfig
* Swap rustfmt configuration to `imports_granularity="Crate"` over deprecated `merge_imports = true`

## [1.2.0]

### Changed

* Update minimum supported rust to 1.56 for `edition 2021`

### Fixed

* Typemap fix: [#282](https://github.com/estk/log4rs/pull/282)

## [1.1.1]

### Added

### Changed

* Removed palaver
* Update `parking_lot` to `0.11`
* Update minimum supported rust to 1.49 for `parking_lot`

### Fixed

* #253

## [1.1.0]

### Added

* Example of compile-time config
* `gettid` for `PatternEncoder`
* Better rotation benchmark statistics
* `tty_only` option to `ConsoleAppender`

### Changed

* Update `arc_swap` to `1.2`
* Update `thread_id` to `4`
* Update docs for `FixedWindow::build`
* Drop `Regex` dependency

### Fixed

* Hide {} in error message from formatting machinery
* Fix link in examples

## [1.0.0]

### Added

* Custom error handling
* Allow parsing of config from string
* Expand env vars in file path of file and RollingFile appenders PR#155
* Console appender can be configured to only write output when it's a TTY

### Changed

* Colors changed to match `env_logger`
* Drop XML config support
* Rename feature `file` to `config_parsing`
* Use `thiserror`/`anyhow` for errors

### Fixed

## [0.13.0]

### Added

### Changed

* Update `serde-xml-rs` to `0.4`
* Update `parking_lot` to `0.11`

### Fixed

* Fix bug where both `pattern_encoder` and `json_encoder` features need to be
  active to use either

## [0.12.0]

### Added

* Derived `Clone` for `Handle`

### Changed

### Fixed

* Build warnings
* Docs typos

## [0.11.0]

A performance issue was discovered with gzip and rolling logs, the
`background_rotation` feature was added to mitigate this by spawning a
background thread to perform the rotation in. Shout out to @yakov-bakhmatov
for the PR!

### Added

* `background_rotation` feature which rotates and compresses log archives in a
  background thread

### Changed

* Deprecate xml feature in preparation for removal
* Simplify and increase visibility of docs
* Swap some synchronization primitives to use `parking_lot` implementations

### Fixed

## [0.10.0]

This is a big  release as we're moving to rust 2018 edition!

### Added

* More badges in the readme

### Changed

* Use rust 2018 edition
* Minimum rust version is 1.38.0
* Update `arcswap`, `serde-value` and `serde-xml-rs`

### Fixed

* Deprecate len method on rolling_file
* Windows build issue after 2018 edition

## [0.9.0]

### Added

* `Logger` is now public
* `PatternEncoder` now has the pid
* Many config structs are now `Clone` and `Debug` for convenience
* JSON logger example added
* File logging example added

### Fixed

* Hierarchical Changelog
* No longer looking for maintainer

## [0.8.3] - 2019-04-02

### Fixed

* Fixed Cargo.toml badge

## [0.8.2] - 2019-04-02

### Changed

* Switched from crossbeam's `ArcCell` to arc-swap's `ArcSwap` internally
* Upgraded toml to 0.5

## [0.8.1] - 2018-10-17

### Added

* Support thread IDs in both JSON and pattern encoders

### Changed

* Upgraded to serde_yaml 0.8

## [0.8.0] - 2017-12-25

### Added

* XML-formatted config files are now supported
* `Append::flush` method

### Changed

* Upgraded to log 0.4

## [0.7.0] - 2017-04-26

### Added

### Changed

* Update to serde 1.0

## [0.6.3] - 2017-04-05

### Added

### Changed

* Fix console appender to actually log to stdout when requested

## [0.6.2] - 2017-03-01

### Added

### Changed

* Fix handling of non-0 bases in rolling file appender

## [0.6.1] - 2017-02-11

### Added

* Add TOML support back in

### Changed

## [0.6.0] - 2017-02-10

### Added

* Enable most features by default. This increases compile times a bit, but is
  way less confusing for people since components aren't randomly missing
* Restructure config deserialization. A log4rs config can now be embedded in
  other config structures and deserialized by downstream users

### Changed

* Update to serde 0.9
* Use serde_derive instead of manual codegen
* Drop TOML support. The toml crate hasn't yet been released with support for
  serde 0.9, but we'll add support back when that lands

## [0.5.2] - 2016-11-25

### Added

* Make Deserializers Clone

### Changed

## [0.5.1] - 2016-11-20

### Added

### Changed

* Update serde_yaml
* Fix file modification time checks in config reloader
