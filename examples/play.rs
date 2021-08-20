// Play a 220 Hertz sine wave through the system's speakers.

use fon::chan::Ch32;
use fon::Frame;
use pasts::Loop;
use std::task::Poll::{self, Pending};
use twang::ops::Gain;
use twang::osc::Sine;
use twang::Synth;
use wavy::{Speakers, SpeakersSink};

type Exit = ();

/// Shared state between tasks on the thread.
struct State {
    /// The chosen set of speakers.
    speakers: Speakers,
    /// A streaming sine wave synthesizer using Twang.
    synth: Synth<Sine, 8>,
}

impl State {
    // Speaker is ready to play more audio.
    fn play(&mut self, player: SpeakersSink) -> Poll<Exit> {
        self.synth.stream(player);
        Pending
    }
}

async fn event_loop() {
    let mut state = State {
        // Connect to the default speakers
        speakers: Speakers::default(),

        // Build "sine wave at 70% amplitude" synthesis algorithm using Twang.
        synth: Synth::new(Sine::new(), |sine, frame: Frame<_, 8>| {
            // Calculate the next sample for each processor
            let sine = sine.next(440.0);
            // Pan the generated audio center
            frame.pan(Gain.next(sine, Ch32::new(0.7)), 0.0)
        }),
    };

    Loop::new(&mut state)
        .when(|s| &mut s.speakers, State::play)
        .await;
}

fn main() {
    pasts::block_on(event_loop());
}
