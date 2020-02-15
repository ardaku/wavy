//! This example records audio and plays it back in real time as it's being
//! recorded.

use std::collections::VecDeque;
use std::io::Write;
use wavy::{Recorder, StereoS16Frame, SampleRate, AudioError};
use pasts::{ThreadInterrupt, Interrupt};

/// Shared data between recorder and player.
struct Shared {
    /// A stereo audio buffer.
    buffer: VecDeque<StereoS16Frame>,
    /// Audio Recorder
    recorder: Recorder,
}

/// Create a new monitor.
async fn monitor() -> Result<(), AudioError> {
    /// Extend buffer by slice of new frames from last plugged in device.
    async fn record(shared: &mut Shared) {
        println!("Recording; running total: @{}", shared.buffer.len());
        let frames = shared.recorder.record_last().await.unwrap();
        shared.buffer.extend(frames);
        println!("Recorded; now: {}", shared.buffer.len());
    }

    let buffer = VecDeque::new();
    println!("Opening recorder…");
    let recorder = Recorder::new(SampleRate::Normal)?;
    println!("Opening player…");
    let mut shared = Shared { buffer, recorder };
    println!("Done, entering async loop…");
    pasts::run!(shared while shared.buffer.len() <= 48_000; record);

    println!("Exited async loop…");

    let mut file = std::fs::File::create("recorded.pcm").unwrap();
    println!("Writing to file…");
    for i in shared.buffer {
        dbg!(i.left());
        file.write(&i.bytes()).unwrap();
    }
    file.flush().unwrap();
    println!("Quitting…");

    Ok(())
}

/// Start the async executor.
fn main() -> Result<(), AudioError> {
    ThreadInterrupt::block_on(monitor())
}
