// Play a 220 Hertz sine wave through the system's speakers.

use fon::{stereo::Stereo32, Sink};
use pasts::{exec, wait};
use twang::{Fc, Signal, Synth};
use wavy::{Speakers, SpeakersSink};

/// An event handled by the event loop.
enum Event<'a> {
    /// Speaker is ready to play more audio.
    Play(SpeakersSink<'a, Stereo32>),
}

/// Shared state between tasks on the thread.
struct State {
    /// A streaming synthesizer using Twang.
    synth: Synth<()>,
}

impl State {
    /// Event loop.  Return false to stop program.
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Play(mut speakers) => speakers.stream(&mut self.synth),
        }
    }
}

/// Program start.
fn main() {
    fn sine(_: &mut (), fc: Fc) -> Signal {
        fc.freq(440.0).sine().gain(0.7)
    }

    let mut state = State {
        synth: Synth::new((), sine),
    };
    let mut speakers = Speakers::default();

    exec!(state.event(wait! {
        Event::Play(speakers.play().await),
    }));
}
