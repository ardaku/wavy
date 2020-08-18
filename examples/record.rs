//! This example records audio and plays it back in real time as it's being
//! recorded.

use fon::{Audio, mono::Mono16};
use pasts::{prelude::*, CvarExec};
use std::cell::RefCell;
use wavy::{Speakers, Microphone, StreamRecv};

/// The program's shared state.
struct State {
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono16>,
}

/// Microphone task (record audio).
async fn microphone_task(state: &RefCell<State>) {
    // Connect to a microphone.
    let mut microphone = Microphone::new().expect("Need a microphone");
    
    loop {
        // 1. Wait for microphone to record some samples.
        let mut stream = microphone.record().await;
        // 2. Borrow shared state mutably.
        let mut state = state.borrow_mut();
        // 3. Write samples into buffer.
        stream.recv(&mut state.buffer);
    }
}

/// Speakers task (play recorded audio).
async fn speakers_task(state: &RefCell<State>, speakers: Speakers<Mono16>) {
    loop {
        // 1. Wait for speaker to need more samples.
        let mut stream = speakers.play().await;
        // 2. Borrow shared state mutably
        let mut state = state.borrow_mut();
        // 3. Generate and write samples into speaker buffer.
        stream.send(&mut state.buffer);
    }
}

/// Program start.
async fn start() {
    // Connect to system's speaker(s)
    let speakers = Speakers::<Mono16>::new();
    // Get the speaker's sample rate.
    let sr = speakers.sample_rate();
    // Initialize shared state.
    let state = RefCell::new(State { buffer: Audio::with_silence(sr, 0) });
    // Create speaker task.
    let mut speakers = speakers_task(&state, speakers);
    // Create microphone task.
    let mut microphone = microphone_task(&state);
    // Wait for first task to complete.
    [speakers.fut(), microphone.fut()].select().await;
}

/// Start the async executor.
fn main() {
    static EXECUTOR: CvarExec = CvarExec::new();
    EXECUTOR.block_on(start())
}
