// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.
//
//! Asynchronous cross-platform real-time audio recording &amp; playback.
//!
//! # Getting Started
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.6"
//! wavy = "0.5"
//! fon = "0.2"
//! ```
//!
//! This example records audio and plays it back in real time as it's being
//! recorded.  (Make sure to wear headphones to avoid feedback):
//!
//! ```rust,no_run
//! use fon::{chan::Ch16, mono::Mono16, Audio, Stream, ops::Src};
//! use pasts::prelude::*;
//! use std::cell::RefCell;
//! use wavy::{Microphone, MicrophoneId, SpeakerId};
//!
//! /// The program's shared state.
//! struct State {
//!     /// Temporary buffer for holding real-time audio samples.
//!     buffer: Audio<Mono16>,
//! }
//!
//! /// Microphone task (record audio).
//! async fn microphone_task(state: &RefCell<State>, mut mic: Microphone<Ch16>) {
//!     loop {
//!         // 1. Wait for microphone to record some samples.
//!         let mut stream = mic.record().await;
//!         // 2. Borrow shared state mutably.
//!         let mut state = state.borrow_mut();
//!         // 3. Write samples into buffer.
//!         state.buffer.extend(&mut stream);
//!     }
//! }
//!
//! /// Speakers task (play recorded audio).
//! async fn speakers_task(state: &RefCell<State>) {
//!     // Connect to system's speaker(s)
//!     let mut speakers = SpeakerId::default().connect::<Mono16>().unwrap();
//!
//!     loop {
//!         // 1. Wait for speaker to need more samples.
//!         let mut sink = speakers.play().await;
//!         // 2. Borrow shared state mutably
//!         let mut state = state.borrow_mut();
//!         // 3. Generate and write samples into speaker buffer.
//!         state.buffer.drain(..).stream(&mut sink, Src);
//!     }
//! }
//!
//! /// Program start.
//! async fn start() {
//!     // Connect to a user-selected microphone.
//!     let microphone = MicrophoneId::default().connect().unwrap();
//!     // Get the microphone's sample rate.
//!     // Initialize shared state.
//!     let state = RefCell::new(State {
//!         buffer: Audio::with_silence(microphone.sample_rate(), 0),
//!     });
//!     // Create speaker and microphone tasks.
//!     task! {
//!         let speakers = speakers_task(&state);
//!         let microphone = microphone_task(&state, microphone)
//!     }
//!     // Wait for first task to complete.
//!     poll![speakers, microphone].await;
//! }
//!
//! /// Start the async executor.
//! fn main() {
//!     exec!(start());
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

#[cfg_attr(target_arch = "wasm32", path = "ffi/wasm/ffi.rs")]
#[cfg_attr(
    not(target_arch = "wasm32"),
    cfg_attr(target_os = "linux", path = "ffi/linux/ffi.rs"),
    cfg_attr(target_os = "android", path = "ffi/android/ffi.rs"),
    cfg_attr(target_os = "macos", path = "ffi/macos/ffi.rs"),
    cfg_attr(target_os = "ios", path = "ffi/ios/ffi.rs"),
    cfg_attr(target_os = "windows", path = "ffi/windows/ffi.rs"),
    cfg_attr(
        any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "bitrig",
            target_os = "openbsd",
            target_os = "netbsd"
        ),
        path = "ffi/bsd/ffi.rs"
    ),
    cfg_attr(target_os = "fuchsia", path = "ffi/fuchsia/ffi.rs"),
    cfg_attr(target_os = "redox", path = "ffi/redox/ffi.rs"),
    cfg_attr(target_os = "none", path = "ffi/none/ffi.rs"),
    cfg_attr(target_os = "dummy", path = "ffi/dummy/ffi.rs")
)]
mod ffi;

mod microphone;
mod route;
mod speaker;

pub use microphone::Microphone;
pub use route::{MicrophoneId, SpeakerId};
pub use speaker::Speaker;
