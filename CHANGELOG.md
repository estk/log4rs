# Change Log

## [Unreleased]

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
