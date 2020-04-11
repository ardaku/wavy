//! This example records audio and plays it back in real time as it's being
//! recorded.

use pasts::{Interrupt, ThreadInterrupt};
use wavy::{Player, Recorder, S16LEx2};

/// Shared data between recorder and player.
struct Shared {
    /// A boolean to indicate whether or not the program is still running.
    running: bool,
    /// A stereo audio buffer.
    buffer: Vec<S16LEx2>,
    /// Audio Recorder
    recorder: Recorder<S16LEx2>,
    /// Audio Player
    player: Player<S16LEx2>,
}

/// Create a new monitor.
async fn monitor() {
    /// Extend buffer by slice of new frames from last plugged in device.
    async fn record(shared: &mut Shared) {
        shared.recorder.record_last(&mut shared.buffer).await;
    }
    /// Drain double ended queue frames into last plugged in device.
    async fn play(shared: &mut Shared) {
        let n_frames = shared.player.play_last(shared.buffer.as_slice()).await;
        shared.buffer.drain(..n_frames.min(shared.buffer.len()));
    }

    let running = true;
    let buffer = Vec::new();
    let recorder = Recorder::new(48_000).unwrap();
    let player = Player::new(48_000).unwrap();
    let mut shared = Shared {
        running,
        buffer,
        recorder,
        player,
    };
    println!("Entering async loop…");
    pasts::tasks!(shared while shared.running; [record, play]);
}

/// Start the async executor.
fn main() {
    ThreadInterrupt::block_on(monitor())
}
