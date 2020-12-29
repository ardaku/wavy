// This example records audio and plays it back in real time as it's being
// recorded.

use fon::{mono::Mono32, Sink, Audio};
use pasts::{exec, wait};
use wavy::{SpeakersId, MicrophoneId, SpeakersSink, MicrophoneStream};

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
    /// Event loop.  Return false to stop program.
    fn event(&mut self, event: Event<'_>) -> bool {
        match event {
            Event::Play(mut speakers) => {
                //println!("Playing");
                speakers.stream(self.buffer.drain())
            },
            Event::Record(microphone) => {
                //println!("Recording");
                self.buffer.extend(microphone);
            },
        }
        true
    }
}

/// Program start.
fn main() {
    let mut state = State { buffer: Audio::with_silence(48_000, 0) };
    let mut speakers = SpeakersId::default().connect().unwrap();
    let mut microphone = MicrophoneId::default().connect().unwrap();

    exec! { state.event( wait! [
        Event::Record(microphone.record().await),
        Event::Play(speakers.play().await),
    ] .await ) }
}
