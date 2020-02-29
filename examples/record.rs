//! This example records audio and plays it back in real time as it's being
//! recorded.

use std::collections::VecDeque;
use wavy::{Player, Recorder, StereoS16Frame, SampleRate, AudioError};
use pasts::{ThreadInterrupt, Interrupt};

/// Shared data between recorder and player.
struct Shared {
    /// A boolean to indicate whether or not the program is still running.
    running: bool,
    /// A stereo audio buffer.
    buffer: VecDeque<StereoS16Frame>,
    /// Audio Recorder
    recorder: Recorder,
    /// Audio Player
    player: Player,
}

/// Create a new monitor.
async fn monitor() -> Result<(), AudioError> {
    /// Extend buffer by slice of new frames from last plugged in device.
    async fn record(shared: &mut Shared) {
        let frames = shared.recorder.record_last().await;
        shared.buffer.extend(frames);
    }
    /// Drain double ended queue frames into last plugged in device.
    async fn play(shared: &mut Shared) {
        let n_frames = shared.player.play_last(shared.buffer.iter()).await;
        shared.buffer.drain(..n_frames.min(shared.buffer.len()));
    }

    let running = true;
    let mut buffer = VecDeque::new();
    buffer.extend([StereoS16Frame::new(0, 0); 1024 * 2].iter());
    let recorder = Recorder::new(SampleRate::Normal)?;
    let player = Player::new(SampleRate::Normal)?;
    let mut shared = Shared { running, buffer, recorder, player };
    println!("Entering async loop…");
    pasts::run!(shared while shared.running; record, play);
    Ok(())
}

/// Start the async executor.
fn main() -> Result<(), AudioError> {
    ThreadInterrupt::block_on(monitor())
}
