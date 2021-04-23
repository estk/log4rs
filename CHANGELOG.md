# Change Log

## [Unreleased]


### New

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

### New

### Changed

* Update `serde-xml-rs` to `0.4`.
* Update `parking_lot` to `0.11`.

### Fixed

* Fix bug where both `pattern_encoder` and `json_encoder` features need to be active to use either.

## [0.12.0]

### New

* Derived `Clone` for `Handle`.

### Changed

### Fixed

* Build warnings
* Docs typos

## [0.11.0]

A performance issue was discovered with gzip and rolling logs, the `background_rotation` feature was
added to mitigate this by spawning a background thread to perform the rotation in. Shout out to @yakov-bakhmatov
for the PR!

### New

* `background_rotation` feature which rotates and compresses log archives in a background thread.

### Changed

* Deprecate xml feature in preparation for removal.
* Simplify and increase visibility of docs.
* Swap some synchronization primitives to use `parking_lot` implementations.

### Fixed


## [0.10.0]

This is a big  release as we're moving to rust 2018 edition!

### New

* More badges in the readme.

### Changed

* Use rust 2018 edition.
* Minimum rust version is 1.38.0
* Update `arcswap`, `serde-value` and `serde-xml-rs`.

### Fixed

* Deprecate len method on rolling_file.
* Windows build issue after 2018 edition.

## [0.9.0]

### New

* `Logger` is now public.
* `PatternEncoder` now has the pid.
* Many config structs are now `Clone` and `Debug` for convenience.
* JSON logger example added.
* File logging example added.

### Fixed

* Hierarchical Changelog
* No longer looking for maintainer.

## [0.8.3] - 2019-04-02

### Fixed

* Fixed Cargo.toml badge.

## [0.8.2] - 2019-04-02

### Changed

* Switched from crossbeam's `ArcCell` to arc-swap's `ArcSwap` internally.
* Upgraded toml to 0.5.

## [0.8.1] - 2018-10-17

### New

* Support thread IDs in both JSON and pattern encoders.

### Changed

* Upgraded to serde_yaml 0.8.

## [0.8.0] - 2017-12-25

### New

* XML-formatted config files are now supported.
* Added the `Append::flush` method.

### Changed

* Upgraded to log 0.4.

## Older

Look at the [release tags] for information about older releases.

[Unreleased]: https://github.com/sfackler/log4rs/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/sfackler/log4rs/compare/v0.8.2...v0.9.0
[0.8.3]: https://github.com/sfackler/log4rs/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/sfackler/log4rs/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/sfackler/log4rs/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/sfackler/log4rs/compare/v0.7.0...v0.8.0
[release tags]: https://github.com/sfackler/log4rs/releases
