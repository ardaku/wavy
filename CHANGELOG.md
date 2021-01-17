# Changelog
All notable changes to `wavy` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

## [0.9.0] - 2021-01-17
### Changed
 - Updated to fon 0.5

## [0.8.1] - 2021-01-08
### Fixed
 - XRUNS on ALSA stopping playback indefinitely.
 - Latency improvements on ALSA.
 - More correct handling of errors for ALSA speaker.

## [0.8.0] - 2020-12-30
### Added
 - Support for stereo audio sources.
 - `MicrophoneStream` concrete type.
 - `SpeakerSink` concrete type.
 - Ability to switch audio type of a speaker or microphone at runtime.
 - `Microphone::supports()` to check if an audio format is available.
 - `Speakers::supports()` to check if an audio format is available.

### Changed
 - Updated to newer `fon` version.
 - Now prefers 32-bit float audio, rather than 16-bit PCM.
 - Rename `Speaker` to `Speakers`

### Removed
 - `MicrophoneId` - now merged into `Microphone`
 - `SpeakerId` - now merged into `Speakers`
 - `Microphone::sample_rate()` - No longer needed
 - `Speaker::sample_rate()` - No longer needed

### Fixed
 - Improved latency on Linux.
 - More consistent behavior across audio devices.
 - Task starving issues are mitigated with new `smelling_salts` dependency.

## [0.7.1] - 2020-12-19
### Fixed
 - Updated `fon` code for wasm and dummy implementation.

## [0.7.0] - 2020-12-19
### Changed
 - Update to new `fon` version 0.3

## [0.6.0] - 2020-11-28
### Added
 - `MicrophoneId`, with `query()` method to get list of microphones.
 - `SpeakerId`, with `query()` method to get list of speakers.

### Changed
 - Rename `Speakers` to `Speaker`
 - Replace `Microphone::new()` with `MicrophoneId::default().connect()`
 - Replace `Speaker::new()` with `SpeakerId::default().connect()`
 - Updated docs and examples to use newest pasts (0.6)

### Fixed
 - Removed a lot of unwraps that occured when a microphone or speaker wasn't
   plugged in.

## [0.5.0] - 2020-11-15
### Changed
 - No longer support stdweb, wasm-pack only (see examples folder)
 - Improved latency on wasm
 - Updated docs and examples to use newest pasts (0.5)

## [0.4.0] - 2020-08-27
### Added
 - `fon` crate dependency for easy resampling and consistency.
 - `Microphone.sample_rate()` for getting the sample rate of a mic
 - `Speakers.sample_rate()` for getting the sample rate of the speakers.

### Changed
 - Renamed `Player` to `Speakers`
 - Renamed `Recorder` to `Microphone`
 - Replaced `Microphone.record_last()` with `Microphone.record()` which returns
   an audio `Stream` instead of taking a mutable reference to a `Vec`.
 - Replaced `Speakers.play_last()` with `Speakers.play()` which returns an audio
   `Sink` instead of passing a slice of samples and returning samples written.
 - `Speakers::new()` now returns `Self` instead of `Option<Self>`

### Removed
 - `S16LEx2` type in favor of `fon` types.

## [0.3.0] - 2020-06-25
### Added
 - Support for running on a Web Page with WASM

### Changed
 - No longer pass in sample rate in `Player::new()` and `Recorder::new()`, it is
   now returned by `.await`ing `Player` or `Record` as an `f64`.

## [0.2.2] - 2020-05-17
### Changed
 - Updated examples / documentation to pasts 0.4

## [0.2.1] - 2020-05-05
### Changed
 - Update documentation to use pasts 0.2

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
