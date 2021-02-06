// This example records audio for 5 seconds and writes to a raw PCM file.

use fon::{mono::Mono32, Audio, Frame};
use pasts::{exec, wait};
use wavy::{Microphone, MicrophoneStream};

/// An event handled by the event loop.
enum Event<'a> {
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
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Record(microphone) => {
                //println!("Recording");
                self.buffer.extend(microphone);
                if self.buffer.len() >= 48_000 * 10 {
                    write_pcm(&self.buffer);
                    std::process::exit(0);
                }
            }
        }
    }
}

/// Save a Raw PCM File from an audio buffer.
fn write_pcm(buffer: &Audio<Mono32>) {
    let mut pcm: Vec<u8> = Vec::new();
    for frame in buffer.iter() {
        let sample: f32 = frame.channels()[0].into();
        pcm.extend(sample.to_le_bytes().iter());
    }
    std::fs::write("pcm.raw", pcm.as_slice()).expect("Failed to write file");
}

/// Program start.
fn main() {
    let mut state = State {
        buffer: Audio::with_silence(48_000, 0),
    };
    let mut microphone = Microphone::default();

    exec!(state.event(wait! {
        Event::Record(microphone.record().await),
    }))
}
