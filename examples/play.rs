// Play a 220 Hertz sine wave through the system's speakers.

use fon::{stereo::Stereo32, Sink};
use pasts::prelude::*;
use twang::{Fc, Signal, Synth};
use wavy::{SpeakersId, SpeakersSink};

/// Shared state between tasks on the thread.
struct State {
    /// A streaming synthesizer using Twang.
    synth: Synth<()>,
}

/// An event handled by the event loop.
enum Event<'a> {
    /// Speaker is ready to play more audio.
    Play(SpeakersSink<'a, Stereo32>),
}

/// Synthesis for sine wave.
fn sine(_: (), fc: Fc) -> Signal {
    fc.freq(440.0).sine().gain(0.7)
}

/// Play audio through the speakers.
fn play(state: &mut State, mut speakers: SpeakersSink<Stereo32>) {
    // Stream samples from `synth` into `speakers`.
    speakers.stream(&mut state.synth);
}

/// Handle an event (the event loop).
fn event(state: &mut State, event: Event) -> bool {
    // Check which event.
    match event {
        // Speaker is ready for more samples, so send them.
        Event::Play(sink) => play(state, sink),
    }
    // Default to not exiting the event loop.
    true
}

/// Program start.
async fn start(mut state: State) {
    // Connect to speakers using Wavy.
    let mut speakers = SpeakersId::default().connect().unwrap();
    // Start event loop over asynchronous tasks.
    while {
        task! { let play = async { Event::Play(speakers.play().await) } };
        event(&mut state, poll![play,].await.1)
    } {}
}

fn main() {
    // Run `start()` in an async executor on it's own thread.
    exec!(start(State {
        synth: Synth::new((), sine)
    }));
}
