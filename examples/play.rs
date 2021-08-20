// Play a 220 Hertz sine wave through the system's speakers.

use fon::Frame;
use fon::chan::Ch32;
use pasts::Loop;
use std::task::Poll;
use twang::{Fc, Signal, Synth};
use wavy::{Speakers, Player};

/// Shared state between tasks on the thread.
struct State {
    /// A streaming synthesizer using Twang.
    synth: Synth<()>,
    ///
    speakers: Speakers,
}

impl State {
    /// Speaker is ready to play more audio.
    fn play(&mut self, mut speakers: Player) -> Poll<()> {
        speakers.stream(&mut self.synth);
        Poll::Pending
    }
}

/// Program start.
async fn run() {
    fn sine(_: &mut (), fc: Fc) -> Signal {
        fc.freq(440.0).sine().gain(0.7)
    }

    let mut state = State {
        synth: Synth::new((), sine),
        speakers: Speakers::default(),
    };

    Loop::new(&mut state)
        .when(|s| &mut s.speakers, State::play)
        .await
}

fn main() {
    pasts::block_on(run())
}
