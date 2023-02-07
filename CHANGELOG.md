# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [1.0.0] - 2023-02-07

### Added

- Enables writing the PDF to standard out

### Changed

- Default to reading from standard input

## [0.1.0] - 2023-02-07

### Added

- First version
- The passphrase can also be read from the `PAPERAGE_PASSPHRASE` environment variable
- Better documentation
- Better logging, including command line flags for verbosity
- Development: added a job summary to the Github Action test job

### Changed

- Renamed the project from ‘Paper Rage’ to ‘PaperAge’
- Better test coverage
- Development: output test summaries and code coverage information in GitHub actions

### Fixed

- Fixed prerelease versioning
- Removed unused dependency (`is-terminal`)
- Prevent accidental overwrites of the output file

<!-- next-url -->
[Unreleased]: https://github.com/crate-ci/cargo-release/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/crate-ci/cargo-release/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/matiaskorhonen/paper-age/compare/v0.1.0-prerelease4...v0.1.0
[0.1.0-prerelease4]: https://github.com/matiaskorhonen/paper-age/releases/tag/b0534db779720e912750d0107b3b03b6551abcdd...v0.1.0-prerelease4
