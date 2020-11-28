//! This example records audio and plays it back in real time as it's being
//! recorded.

use fon::{chan::Ch16, mono::Mono16, Audio, Stream};
use pasts::prelude::*;
use std::cell::RefCell;
use wavy::{Microphone, Speaker};

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
    let mut speakers = Speaker::<Mono16>::new();

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
    // Create speaker and microphone tasks.
    task! {
        let speakers = speakers_task(&state);
        let microphone = microphone_task(&state, microphone)
    }
    // Wait for first task to complete.
    poll![speakers, microphone].await;
}

/// Start the async executor.
fn main() {
    exec!(start());
}
