//! Cross-platform real-time audio recording &amp; playback.
//!
//! The sound waves are _so_ wavy!
//!
//! # Getting Started
//! This example records audio and plays it back in real time as it's being recorded.  (Make sure to
//! wear headphones to avoid feedback).
//!
//! ```rust,no_run
//! use wavy::*;
//!
//! use std::collections::VecDeque;
//!
//! fn main() -> Result<(), AudioError> {
//!     // Connect to the speaker and microphone systems.
//!     let mut mic = MicrophoneSystem::new(SampleRate::Normal)?;
//!     let mut speaker = SpeakerSystem::new(SampleRate::Normal)?;
//!
//!     let mut buffer = VecDeque::new();
//!
//!     loop {
//!         // Record some sound.
//!         mic.record(&mut |_whichmic, l, r| {
//!             buffer.push_back((l, r));
//!         });
//!
//!         // Play that sound.
//!         speaker.play(&mut || {
//!             if let Some((lsample, rsample)) = buffer.pop_front() {
//!                 AudioSample::stereo(lsample, rsample)
//!             } else {
//!                 // Play silence if not enough has been recorded yet.
//!                 AudioSample::stereo(0, 0)
//!             }
//!         });
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![doc(
	html_logo_url = "http://free.plopgrizzly.com/plop/icon.svg",
	html_favicon_url = "http://free.plopgrizzly.com/plop/icon.svg",
)]

mod error;
mod sample_rate;
mod system;
// mod resampler;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

extern crate libc;

pub use error::AudioError;
pub use sample_rate::SampleRate;
pub use system::AudioSample;
pub use system::MicrophoneSystem;
pub use system::SpeakerSystem;
