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
//!     buffer: VecDeque<S16LEx2>,
//!     /// Audio Recorder
//!     recorder: Recorder,
//!     /// Audio Player
//!     player: Player<S16LEx2>,
//! }
//!
//! /// Create a new monitor.
//! async fn monitor() {
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
//!     let recorder = Recorder::new(48_000).unwrap();
//!     let player = Player::new(48_000).unwrap();
//!     let mut shared = Shared { running, buffer, recorder, player };
//!     pasts::tasks!(shared while shared.running; [record, play]);
//! }
//!
//! /// Start the async executor.
//! fn main() {
//!     ThreadInterrupt::block_on(monitor())
//! }
//! ```

#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://libcala.github.io/logo.svg",
    html_favicon_url = "https://libcala.github.io/icon.svg"
)]
#![deny(unsafe_code)]

// mod system;
mod stereo;
// mod resampler;
mod frame;
mod player;
mod recorder;

#[cfg_attr(target_arch = "wasm32", path = "ffi/wasm32.rs")]
#[cfg_attr(
    not(target_arch = "wasm32"),
    cfg_attr(target_os = "linux", path = "ffi/linux.rs"),
    cfg_attr(target_os = "android", path = "ffi/android.rs"),
    cfg_attr(target_os = "macos", path = "ffi/macos.rs"),
    cfg_attr(target_os = "ios", path = "ffi/ios.rs"),
    cfg_attr(target_os = "windows", path = "ffi/windows.rs"),
    cfg_attr(
        any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "bitrig",
            target_os = "openbsd",
            target_os = "netbsd"
        ),
        path = "ffi/bsd.rs"
    ),
    cfg_attr(target_os = "fuchsia", path = "ffi/fuchsia.rs"),
    cfg_attr(target_os = "redox", path = "ffi/redox.rs"),
    cfg_attr(target_os = "none", path = "ffi/none.rs"),
    cfg_attr(target_os = "dummy", path = "ffi/dummy.rs"),
)]
mod ffi;

pub use player::Player;
pub use recorder::Recorder;
pub use stereo::S16LEx2;
