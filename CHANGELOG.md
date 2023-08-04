# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `detect_language` option.

## [0.3.0]

### Added
- Derive `Serialize` for all response types.

### Fixed
- Use the users builder options when building a streaming URL.
- Make sure that `Future` returned from `StreamRequestBuilder::start()` is `Send`.

### Changed
- Use Rustls instead of OpenSSL.

[Unreleased]: https://github.com/deepgram-devs/deepgram-rust-sdk/compare/0.3.0...HEAD
[0.3.0]: https://github.com/deepgram-devs/deepgram-rust-sdk/compare/0.2.1...0.3.0
