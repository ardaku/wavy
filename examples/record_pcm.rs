//! This example records audio and plays it back in real time as it's being
//! recorded.  Examples are in the public domain.

use pasts::{CvarExec, prelude::*};
use std::io::Write;
use wavy::{Recorder, S16LEx2};

/// Shared data between recorder and player.
struct Shared {
    /// A stereo audio buffer.
    buffer: Vec<S16LEx2>,
    /// Audio Recorder
    recorder: Recorder<S16LEx2>,
}

/// Create a new monitor.
async fn monitor() {
    let buffer = vec![];
    println!("Opening recorder…");
    let recorder = Recorder::new().unwrap();
    println!("Opening player…");
    let mut shared = Shared { buffer, recorder };
    println!("Done, entering async loop…");
    while shared.buffer.len() <= 48_000 * 10 {
        println!("Recording; running total: @{}", shared.buffer.len());
        shared.recorder.fut().await;
        shared.recorder.record_last(&mut shared.buffer);
        println!("Recorded; now: {}", shared.buffer.len());
    }
    println!("Exited async loop…");

    let mut file = std::fs::File::create("recorded.pcm").unwrap();
    println!("Writing to file…");
    for i in shared.buffer {
        dbg!(i.left());
        file.write(&i.bytes()).unwrap();
    }
    file.flush().unwrap();
    println!("Quitting…");
}

/// Start the async executor.
fn main() {
    static EXECUTOR: CvarExec = CvarExec::new();

    EXECUTOR.block_on(monitor())
}
