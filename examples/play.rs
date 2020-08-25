//! Play a 220 Hertz sine wave through the system's speakers.

use fon::mono::Mono64;
use pasts::{prelude::*, CvarExec};
use std::cell::RefCell;
use twang::Synth;
use wavy::Speakers;

/// The program's shared state.
struct State {}

/// Speakers task (play sine wave).
async fn speakers(state: &RefCell<State>) {
    // Connect to system's speaker(s)
    let mut speakers = Speakers::<Mono64>::new();
    // Create a new synthesizer
    let mut synth = Synth::new();

    loop {
        // 1. Wait for speaker to need more samples.
        let sink = speakers.play().await;
        // 2. Borrow shared state mutably
        let _state = state.borrow_mut();
        // 3. Generate and write samples into speaker buffer.
        synth.gen(sink, |fc| fc.freq(440.0).sine().amp(0.7));
    }
}

/// Program start.
async fn start() {
    // Initialize shared state.
    let state = RefCell::new(State {});
    // Create speaker task.
    let mut speakers = speakers(&state);
    // Wait for first task to complete.
    [speakers.fut()].select().await;
}

/// Start the async executor.
fn main() {
    static EXECUTOR: CvarExec = CvarExec::new();
    EXECUTOR.block_on(start())
}
