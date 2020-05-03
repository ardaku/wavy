# Changelog
All notable changes to `wavy` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

## [0.2.0] - 2020-05-03
### Added
- Async/Await support

### Removed
- `SampleRate` enum, you can now specify whatever you like in hertz.
- `AudioError` enum, since there was only one error anyway (device not found),
  all occurances were replaced with `Option`.

### Changed
- Rename `MicrophoneSystem` to `Recorder`.
- Rename `SpeakerSystem` to `Player`.
- `AudioSample` was replaced with `S16LEx2`.  More audio formats will be added
  in the future.

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
