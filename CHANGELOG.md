# Changelog
<!-- We follow the Keep a Changelog standard https://keepachangelog.com/en/1.0.0/ -->

## [Unreleased]
### Added
- [#34](https://github.com/tweag/genealogos/pull/34) implements a web-based GUI for Genealogos
- [#36](https://github.com/tweag/genealogos/pull/36) include nixtract's new narinfo information
- [#38](https://github.com/tweag/genealogos/pull/38) display nixtract's status information when running
- [#44](https://github.com/tweag/genealogos/pull/44) adds two functions to the `Backend` trait to set options

### Changed
- [#41](https://github.com/tweag/genealogos/pull/41) reworked the Genealogos fronend, paving the way for supporting other bom formats
- [#46](https://github.com/tweag/genealogos/pull/46) replaces `Source::Flake` with `Source::Installable` and `--flake-ref, --attribute-path` with `--installable`
