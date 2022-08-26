// This example records audio and plays it back in real time as it's being
// recorded.

// Setup async main
include!(concat!(env!("OUT_DIR"), "/main.rs"));

use fon::{mono::Mono32, Audio, Sink};
use pasts::{prelude::*, Join};
use wavy::{Microphone, MicrophoneStream, Speakers, SpeakersSink};

/// Shared state between tasks on the thread.
struct App<'a> {
    /// Handle to speakers
    speakers: &'a mut Speakers<1>,
    /// Handle to the microphone
    microphone: &'a mut Microphone<1>,
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono32>,
}

impl App<'_> {
    /// Speaker is ready to play more audio.
    fn play(&mut self, mut sink: SpeakersSink<Mono32>) -> Poll<()> {
        sink.stream(self.buffer.drain());
        Pending
    }

    /// Microphone has recorded some audio.
    fn record(&mut self, stream: MicrophoneStream<Mono32>) -> Poll<()> {
        self.buffer.extend(stream);
        Pending
    }

    /// Program start.
    async fn main(_executor: Executor) {
        let speakers = &mut Speakers::default();
        let microphone = &mut Microphone::default();
        let buffer = Audio::with_silence(48_000, 0);
        let mut app = App {
            speakers,
            microphone,
            buffer,
        };

        Join::new(&mut app)
            .on(|s| s.speakers, App::play)
            .on(|s| s.microphone, App::record)
            .await
    }
}
