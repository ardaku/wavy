//! Asynchronous cross-platform real-time audio recording &amp; playback.
//!
//! The sound waves are _so_ wavy!
//!
//! # Getting Started
//! This example records audio and plays it back in real time as it's being
//! recorded.  (Make sure to wear headphones to avoid feedback).
//!
//! ```rust,no_run
//! use wavy::*;
//! use std::collections::VecDeque;
//! use pasts::{ThreadInterrupt, Interrupt};
//!
//! /// Shared data between recorder and player.
//! struct Shared {
//!     /// A boolean to indicate whether or not the program is still running.
//!     running: bool,
//!     /// A stereo audio buffer.
//!     buffer: VecDeque<StereoS16>,
//!     /// Audio Recorder
//!     recorder: Recorder,
//!     /// Audio Player
//!     player: Player,
//! }
//!
//! /// Create a new monitor.
//! async fn monitor() -> Result<(), AudioError> {
//!     /// Extend buffer by slice of new frames from last plugged in device.
//!     async fn record(shared: &mut Shared) {
//!         let frames = shared.recorder.record_last().await;
//!         shared.buffer.extend(frames);
//!     }
//!     /// Drain double ended queue frames into last plugged in device.
//!     async fn play(shared: &mut Shared) {
//!         let n_frames = shared.player.play_last(shared.buffer.iter()).await;
//!         shared.buffer.drain(..n_frames.min(shared.buffer.len()));
//!     }
//!
//!     let running = true;
//!     let buffer = VecDeque::new();
//!     let recorder = Recorder::new(SampleRate::Normal)?;
//!     let player = Player::new(SampleRate::Normal)?;
//!     let mut shared = Shared { running, buffer, recorder, player };
//!     pasts::tasks!(shared while shared.running; [record, play]);
//!     Ok(())
//! }
//!
//! /// Start the async executor.
//! fn main() -> Result<(), AudioError> {
//!     ThreadInterrupt::block_on(monitor())
//! }
//! ```

#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://libcala.github.io/logo.svg",
    html_favicon_url = "https://libcala.github.io/icon.svg"
)]
#![deny(unsafe_code)]

mod error;
mod sample_rate;
// mod system;
mod stereo;
// mod resampler;
mod player;
mod recorder;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
use linux as ffi;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
use wasm as ffi;

#[cfg(target_os = "macos")]
mod apple;
#[cfg(target_os = "macos")]
use apple as ffi;

pub use error::AudioError;
pub use player::Player;
pub use recorder::Recorder;
pub use sample_rate::SampleRate;
pub use stereo::StereoS16;
