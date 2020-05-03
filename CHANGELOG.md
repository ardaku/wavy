# Changelog
All notable changes to `wavy` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

## [0.2.0] - 2020-05-03 (Unreleased)
### Added
- Async/Await support

### Removed
- 

### Changed
- Rename `MicrophoneSystem` to `MicrophoneList`.
- Rename `SpeakerSystem` to `SpeakerList`.

### Fixed
- Undefined behavior using `std::mem::uninitialized()` by replacing with
  `std::mem::MaybeUninit()`.

## [0.1.2] - 2019-05-13
### Fixed
- Broken links.

## [0.1.1] - 2019-05-13
### Fixed
- `cargo-clippy` warnings.

## [0.1.0] - 2019-03-21
### Added
- Audio playback support on Linux.
- Audio recording support on Linux.
