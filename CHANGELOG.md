# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Changed

- Documentation fixes and improvements
- Minor dependency updates

## [1.1.0] - 2023-02-14

### Added

- Support for the letter paper size
- Shell completion scripts for Bash, Fish, and Zsh

### Fixed

- The man page is now included in the release archives

## [1.0.1] - 2023-02-07

### Added

- PaperAge can now be installed via a [Homebrew Tap](https://github.com/matiaskorhonen/paper-age#homebrew)

### Fixed

- Documentation fixes

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
[Unreleased]: https://github.com/matiaskorhonen/paper-age/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/matiaskorhonen/paper-age/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/matiaskorhonen/paper-age/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/matiaskorhonen/paper-age/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/matiaskorhonen/paper-age/compare/v0.1.0-prerelease4...v0.1.0
[0.1.0-prerelease4]: https://github.com/matiaskorhonen/paper-age/releases/tag/b0534db779720e912750d0107b3b03b6551abcdd...v0.1.0-prerelease4
