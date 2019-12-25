# Wavy

[![docs.rs](https://docs.rs/wavy/badge.svg)](https://docs.rs/wavy)
[![build status](https://api.travis-ci.com/libcala/wavy.svg?branch=master)](https://travis-ci.com/libcala/wavy)
[![crates.io](https://img.shields.io/crates/v/wavy.svg)](https://crates.io/crates/wavy)
[![Zulip Chat](https://img.shields.io/badge/zulip-join_chat-darkgreen.svg)](https://cala.zulipchat.com/join/wkdkw53xb5htnchg8kqz0du0/)

[About](https://libcala.github.io/wavy) |
[Source](https://github.com/libcala/wavy) |
[Changelog](https://libcala.github.io/wavy/changelog)

# About

Asynchronous cross-platform real-time audio recording &amp; playback.

The sound waves are _so_ wavy!

# Getting Started
This example records audio and plays it back in real time as it's being
recorded.  (Make sure to wear headphones to avoid feedback).

```rust
use wavy::*;

use std::collections::VecDeque;

fn main() -> Result<(), AudioError> {
    // Connect to the speakers and microphones.
    let mut mic = MicrophoneList::new(SampleRate::Normal)?;
    let mut speaker = SpeakerList::new(SampleRate::Normal)?;

    let mut buffer = VecDeque::new();

    loop {
        // Record some sound.
        mic.record(&mut |_whichmic, l, r| {
            buffer.push_back((l, r));
        });

        // Play that sound.
        speaker.play(&mut || {
            if let Some((lsample, rsample)) = buffer.pop_front() {
                AudioSample::stereo(lsample, rsample)
            } else {
                // Play silence if not enough has been recorded yet.
                AudioSample::stereo(0, 0)
            }
        });
    }
}
```

## Features
- Linux (ALSA) support.
- Microphone audio recording.
- Speaker audio playback.

## TODO
- Audio channel mixing.
- Windows support.
- MacOS and iOS support.
- WASM support.
- Test on Android.
- Nintendo Switch support (And other game consoles).
- Sound from specific direction (Radians) and volume for video games.
- Surround sound 5.1 support.
- Audio Resampling.

# Contributing
Contributors are always welcome!  Whether it is a bug report, bug fix, feature
request, feature implementation or whatever.  Don't be shy about getting
involved.  I always make time to fix bugs, so usually a patched version of the
library will be out soon after a report.  Features take me longer, though.  I'll
also always listen to any design critiques you have.  If you have any questions
you can email me at jeronlau@plopgrizzly.com.  Otherwise, here's a link to the
[issues on GitHub](https://github.com/libcala/wavy/issues).

And, as always, make sure to always follow the
[code of conduct](https://github.com/libcala/wavy/blob/master/CODEOFCONDUCT.md).
Happy coding!

# License
This repository is licensed under either of the following:

- MIT License (MIT) - See accompanying file
  [LICENSE_MIT.txt](https://github.com/libcala/wavy/blob/master/LICENSE_MIT.txt)
  or copy at https://opensource.org/licenses/MIT
- Boost Software License (BSL-1.0) - See accompanying file
  [LICENSE_BSL.txt](https://github.com/libcala/wavy/blob/master/LICENSE_BSL.txt)
  or copy at https://www.boost.org/LICENSE_1_0.txt

at your option.

## Contribution Licensing
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above without any
additional terms or conditions.
