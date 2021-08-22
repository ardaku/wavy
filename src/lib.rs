// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! Asynchronous cross-platform real-time audio recording &amp; playback.
//!
//! # Getting Started
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.7"
//! wavy = "0.8"
//! fon = "0.4"
//! ```
//!
//! This example records audio and plays it back in real time as it's being
//! recorded.  (Make sure to wear headphones to avoid feedback):
//!
//! ```rust,no_run
//! use fon::{stereo::Stereo32, Sink, Audio};
//! use pasts::{exec, wait};
//! use wavy::{Speakers, Microphone, SpeakersSink, MicrophoneStream};
//!
//! /// An event handled by the event loop.
//! enum Event<'a> {
//!     /// Speaker is ready to play more audio.
//!     Play(SpeakersSink<'a, Stereo32>),
//!     /// Microphone has recorded some audio.
//!     Record(MicrophoneStream<'a, Stereo32>),
//! }
//!
//! /// Shared state between tasks on the thread.
//! struct State {
//!     /// Temporary buffer for holding real-time audio samples.
//!     buffer: Audio<Stereo32>,
//! }
//!
//! impl State {
//!     /// Event loop.  Return false to stop program.
//!     fn event(&mut self, event: Event<'_>) {
//!         match event {
//!             Event::Play(mut speakers) => speakers.stream(self.buffer.drain()),
//!             Event::Record(microphone) => self.buffer.extend(microphone),
//!         }
//!     }
//! }
//!
//! /// Program start.
//! fn main() {
//!     let mut state = State { buffer: Audio::with_silence(48_000, 0) };
//!     let mut speakers = Speakers::default();
//!     let mut microphone = Microphone::default();
//!
//!     exec!(state.event(wait! {
//!         Event::Record(microphone.record().await),
//!         Event::Play(speakers.play().await),
//!     }));
//! }
//! ```

#![doc(
    html_logo_url = "https://libcala.github.io/logo.svg",
    html_favicon_url = "https://libcala.github.io/icon.svg",
    html_root_url = "https://docs.rs/wavy"
)]
#![deny(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

mod consts;
mod env;
mod listener;
mod microphone;
mod speakers;

pub use listener::Listener;
pub use microphone::{Microphone, Recorder};
pub use speakers::{Player, Speakers};
