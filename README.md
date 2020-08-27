# Wavy

#### Asynchronous cross-platform real-time audio recording &amp; playback.

[![Build Status](https://api.travis-ci.org/libcala/wavy.svg?branch=master)](https://travis-ci.org/libcala/wavy)
[![Docs](https://docs.rs/wavy/badge.svg)](https://docs.rs/wavy)
[![crates.io](https://img.shields.io/crates/v/wavy.svg)](https://crates.io/crates/wavy)

The sound waves are _so_ wavy!  Wavy supports microphone audio recording and
speaker audio playback using S16LEx2 audio format for these platforms:

### Platforms
- Linux (Using ALSA)
- Web (Compile to Web Assembly)

### Planned Platforms
- Windows
- MacOS and iOS
- BSD
- Fuchsia
- Redox
- Android (might already work)
- Nintendo Switch (and other game consoles)
- WASI
- Others (Make a PR if you think of one)

### Planned Capabilities
- Audio channel mixing.
- Audio Resampling.
- Surround sound 5.1 support.
- Sound from specific direction (Radians) and volume for video games.

## Table of Contents
- [Getting Started](#getting-started)
   - [Example](#example)
   - [API](#api)
   - [Features](#features)
- [Upgrade](#upgrade)
- [License](#license)
   - [Contribution](#contribution)

## Getting Started
Add the following to your `Cargo.toml`.

```toml
[dependencies]
pasts = "0.4"
wavy = "0.4"
fon = "0.2"
```

### Example
This example records audio and plays it back in real time as it's being
recorded.  (Make sure to wear headphones to avoid feedback).

```rust,no_run
use fon::{chan::Ch16, mono::Mono16, Audio, Stream};
use pasts::{prelude::*, CvarExec};
use std::cell::RefCell;
use wavy::{Microphone, Speakers};

/// The program's shared state.
struct State {
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono16>,
}

/// Microphone task (record audio).
async fn microphone_task(state: &RefCell<State>, mut mic: Microphone<Ch16>) {
    loop {
        // 1. Wait for microphone to record some samples.
        let mut stream = mic.record().await;
        // 2. Borrow shared state mutably.
        let mut state = state.borrow_mut();
        // 3. Write samples into buffer.
        state.buffer.extend(&mut stream);
    }
}

/// Speakers task (play recorded audio).
async fn speakers_task(state: &RefCell<State>) {
    // Connect to system's speaker(s)
    let mut speakers = Speakers::<Mono16>::new();

    loop {
        // 1. Wait for speaker to need more samples.
        let mut sink = speakers.play().await;
        // 2. Borrow shared state mutably
        let mut state = state.borrow_mut();
        // 3. Generate and write samples into speaker buffer.
        state.buffer.drain(..).stream(&mut sink);
    }
}

/// Program start.
async fn start() {
    // Connect to a user-selected microphone.
    let microphone = Microphone::new().expect("Need a microphone");
    // Get the microphone's sample rate.
    // Initialize shared state.
    let state = RefCell::new(State {
        buffer: Audio::with_silence(microphone.sample_rate(), 0),
    });
    // Create speaker task.
    let mut speakers = speakers_task(&state);
    // Create microphone task.
    let mut microphone = microphone_task(&state, microphone);
    // Wait for first task to complete.
    [speakers.fut(), microphone.fut()].select().await;
}

/// Start the async executor.
fn main() {
    static EXECUTOR: CvarExec = CvarExec::new();
    EXECUTOR.block_on(start())
}
```

### API
API documentation can be found on [docs.rs](https://docs.rs/wavy).

### Features
There are no optional features.

## Upgrade
You can use the
[changelog](https://github.com/libcala/wavy/blob/master/CHANGELOG.md)
to facilitate upgrading this crate as a dependency.

## License
Licensed under either of
 - Apache License, Version 2.0,
   ([LICENSE-APACHE](https://github.com/libcala/wavy/blob/master/LICENSE-APACHE) or
   [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
 - Zlib License,
   ([LICENSE-ZLIB](https://github.com/libcala/wavy/blob/master/LICENSE-ZLIB) or
   [https://opensource.org/licenses/Zlib](https://opensource.org/licenses/Zlib))

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Contributors are always welcome (thank you for being interested!), whether it
be a bug report, bug fix, feature request, feature implementation or whatever.
Don't be shy about getting involved.  I always make time to fix bugs, so usually
a patched version of the library will be out a few days after a report.
Features requests will not complete as fast.  If you have any questions, design
critques, or want me to find you something to work on based on your skill level,
you can email me at [jeronlau@plopgrizzly.com](mailto:jeronlau@plopgrizzly.com).
Otherwise,
[here's a link to the issues on GitHub](https://github.com/libcala/wavy/issues).
Before contributing, check out the
[contribution guidelines](https://github.com/libcala/wavy/blob/master/CONTRIBUTING.md),
and, as always, make sure to follow the
[code of conduct](https://github.com/libcala/wavy/blob/master/CODE_OF_CONDUCT.md).
