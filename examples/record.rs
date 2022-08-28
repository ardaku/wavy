// This example records audio for 5 seconds and writes to a raw PCM file.

// Setup async main
include!(concat!(env!("OUT_DIR"), "/main.rs"));

use fon::{mono::Mono32, Audio, Frame};
use pasts::{prelude::*, Join};
use wavy::{Microphone, MicrophoneStream};

/// Shared state between tasks on the thread.
struct App<'a> {
    /// Handle to the mono microphone
    microphone: &'a mut Microphone<1>,
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono32>,
}

impl App<'_> {
    /// Event loop.  Return false to stop program.
    fn record(&mut self, stream: MicrophoneStream<Mono32>) -> Poll<()> {
        self.buffer.extend(stream);
        if self.buffer.len() >= 48_000 * 10 {
            return Ready(());
        }
        Pending
    }

    async fn main(_executor: Executor) {
        let buffer = Audio::with_silence(48_000, 0);
        let microphone = &mut Microphone::default();
        let mut app = App { buffer, microphone };

        Join::new(&mut app).on(|s| s.microphone, App::record).await;

        write_pcm(&app.buffer);
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
