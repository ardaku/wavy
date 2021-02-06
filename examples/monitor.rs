// This example records audio and plays it back in real time as it's being
// recorded.

use fon::{mono::Mono32, Audio, Sink};
use pasts::exec;
use wavy::{Microphone, MicrophoneStream, Speakers, SpeakersSink};

/// An event handled by the event loop.
enum Event<'a> {
    /// Speaker is ready to play more audio.
    Play(SpeakersSink<'a, Mono32>),
    /// Microphone has recorded some audio.
    Record(MicrophoneStream<'a, Mono32>),
}

/// Shared state between tasks on the thread.
struct State {
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono32>,
}

impl State {
    /// Event loop.
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Play(mut speakers) => speakers.stream(self.buffer.drain()),
            Event::Record(microphone) => self.buffer.extend(microphone),
        }
    }
}

/// Program start.
fn main() {
    let mut state = State {
        buffer: Audio::with_silence(48_000, 0),
    };
    let mut speakers = Speakers::default();
    let mut microphone = Microphone::default();

    exec!(state.event(pasts::wait! {
        Event::Play(speakers.play().await),
        Event::Record(microphone.record().await),
    }))
}
