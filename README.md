# Wavy
Cross-platform real-time audio recording &amp; playback.

The sound waves are _so_ wavy!

# Getting Started
This example records audio and plays it back in real time as it's being recorded.  (Make sure to
wear headphones to avoid feedback).

```rust
use wavy::*;

use std::collections::VecDeque;

fn main() -> Result<(), AudioError> {
    // Connect to the speaker and microphone systems.
    let mut mic = MicrophoneSystem::new(SampleRate::Normal)?;
    let mut speaker = SpeakerSystem::new(SampleRate::Normal)?;

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
* Linux (ALSA) support.
* Microphone audio recording.
* Speaker audio playback.

## TODO
* Audio channel mixing.
* Windows support.
* MacOS support.
* Sound from specific direction (Radians) and volume for video games.
* Surround sound 5.1 support.
* Audio Resampling.

## Links
* [Website](https://jeronaldaron.plopgrizzly.com/wavy)
* [Cargo](https://crates.io/crates/wavy)
* [Documentation](https://docs.rs/wavy)
* [Change Log](https://jeronaldaron.plopgrizzly.com/wavy/changelog)
* [Contributors](https://jeronaldaron.plopgrizzly.com/wavy/contributors)
* [Code of Conduct](https://jeronaldaron.plopgrizzly.com/wavy/codeofconduct)
