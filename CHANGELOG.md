# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Changed

- Adds codesigning and notarization on macOS
- Minor dependency updates

## [1.3.2] - 2024-07-01

### Changed

- Minor dependency updates

## [1.3.1] - 2024-06-07

### Changed

- Added [artifact attestations](https://docs.github.com/en/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds)
- Updates the time crate to fix build issues with Rust nightly

## [1.3.0] - 2024-05-01

### Changed

- Updated age to v0.10.0
- Use rpassword directly for passphrase prompt
- Removes indirect dependency on atty (GHSA-g98v-hv3f-hcfr)
- Other minor dependency updates

## [1.2.1] - 2024-01-27

### Changed

- Minor dependency updates

## [1.2.0] - 2023-10-21

### Added

- The passphrase/notes field below the QR code is now configurable
- Use font subsetting to reduce the file size of the output PDFs

### Changed

- Update rustix to v0.38.20 (GHSA-c827-hfw6-qwvm)
- Other minor dependency updates

## [1.1.4] - 2023-10-12

### Changed

- Update printpdf to v0.6.0
- Other minor dependency updates

## [1.1.3] - 2023-07-10

### Changed

- Update the age crate to v0.9.2
- Other minor dependency updates

## [1.1.2] - 2023-04-25

### Changed

- Minor dependency updates

## [1.1.1] - 2023-03-08

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
[Unreleased]: https://github.com/matiaskorhonen/paper-age/compare/v1.3.2...HEAD
[1.3.2]: https://github.com/matiaskorhonen/paper-age/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/matiaskorhonen/paper-age/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/matiaskorhonen/paper-age/compare/v1.2.1...v1.3.0
[1.2.1]: https://github.com/matiaskorhonen/paper-age/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/matiaskorhonen/paper-age/compare/v1.1.4...v1.2.0
[1.1.4]: https://github.com/matiaskorhonen/paper-age/compare/v1.1.3...v1.1.4
[1.1.3]: https://github.com/matiaskorhonen/paper-age/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/matiaskorhonen/paper-age/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/matiaskorhonen/paper-age/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/matiaskorhonen/paper-age/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/matiaskorhonen/paper-age/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/matiaskorhonen/paper-age/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/matiaskorhonen/paper-age/compare/v0.1.0-prerelease4...v0.1.0
[0.1.0-prerelease4]: https://github.com/matiaskorhonen/paper-age/releases/tag/b0534db779720e912750d0107b3b03b6551abcdd...v0.1.0-prerelease4
