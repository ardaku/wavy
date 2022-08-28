// Play a 220 Hertz sine wave through the system's speakers.

// Setup async main
include!(concat!(env!("OUT_DIR"), "/main.rs"));

use fon::{stereo::Stereo32, Sink};
use pasts::{prelude::*, Join};
use twang::{Fc, Signal, Synth};
use wavy::{Speakers, SpeakersSink};

/// Shared state between tasks on the thread.
struct App {
    /// Handle to stereo speakers
    speakers: Speakers<2>,
    /// A streaming synthesizer using Twang.
    synth: Synth<()>,
}

impl App {
    /// Speaker is ready to play more audio.
    fn play(&mut self, mut sink: SpeakersSink<Stereo32>) -> Poll<()> {
        sink.stream(&mut self.synth);
        Pending
    }

    /// Program start.
    async fn main(_executor: Executor) {
        fn sine(_: &mut (), fc: Fc) -> Signal {
            fc.freq(440.0).sine().gain(0.7)
        }

        let speakers = Speakers::default();
        let synth = Synth::new((), sine);
        let mut app = App { speakers, synth };

        Join::new(&mut app).on(|s| &mut s.speakers, App::play).await;
    }
}
