//! This example records audio and plays it back in real time as it's being recorded.

use wavy::prelude::*;

use std::collections::VecDeque;

fn main() -> Result<(), AudioError> {
    println!("Opening microphone system");
    let mut mic = MicrophoneSystem::new(wavy::SampleRate::Normal)?;

    println!("Opening speaker system");
    let mut speaker = SpeakerSystem::new(wavy::SampleRate::Normal)?;

    println!("Done");

    let mut buffer = VecDeque::new();

    loop {
        mic.record(&mut |_index, l, r| {
            buffer.push_back((l, r));
        });

        speaker.play(&mut || {
            if let Some((lsample, rsample)) = buffer.pop_front() {
                (lsample, rsample)
            } else {
                (0, 0)
            }
        });
    }
}
