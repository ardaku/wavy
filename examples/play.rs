// Play a 220 Hertz sine wave through the system's speakers.

use pasts::{exec, wait};
use twang::{Synth};
use twang::osc::Sine;
use twang::ops::Gain;
use wavy::{Speakers, SpeakersSink};
use fon::Frame;
use fon::chan::Ch32;

/// An event handled by the event loop.
enum Event<'a> {
    /// Speaker is ready to play more audio.
    Play(SpeakersSink<'a, 2>),
}

// State of the synthesizer.
struct Processors {
    sine: Sine,
}

/// Shared state between tasks on the thread.
struct State {
    /// A streaming synthesizer using Twang.
    synth: Synth<Processors, 2>,
}

impl State {
    /// Event loop.  Return false to stop program.
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Play(speakers) => self.synth.stream(speakers),
        }
    }
}

/// Program start.
fn main() {
    // Create audio processors
    let proc = Processors { sine: Sine::new() };
    // Build synthesis algorithm
    let synth = Synth::new(proc, |proc, frame: Frame<_, 2>| {
        // Calculate the next sample for each processor
        let sine = proc.sine.next(440.0);
        // Pan the generated audio center
        frame.pan(Gain.next(sine, Ch32::new(0.7)), 0.0)
    });
    // Connect to the default speakers.
    let mut speakers = Speakers::default();
    // 
    let mut state = State {
        synth,
    };

    exec!(state.event(wait! {
        Event::Play(speakers.play().await),
    }));
}
