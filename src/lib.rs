// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
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
//! pasts = "0.12"
//! wavy = "0.10"
//! fon = "0.5"
//! ```
//!
//! This example records audio and plays it back in real time as it's being
//! recorded.  (Make sure to wear headphones to avoid feedback):
//!
//! ```rust
//! use fon::{mono::Mono32, Audio, Sink};
//! use pasts::{prelude::*, Join};
//! use wavy::{Microphone, MicrophoneStream, Speakers, SpeakersSink};
//!
//! /// Shared state between tasks on the thread.
//! struct App<'a> {
//!     /// Handle to speakers
//!     speakers: &'a mut Speakers<1>,
//!     /// Handle to the microphone
//!     microphone: &'a mut Microphone<1>,
//!     /// Temporary buffer for holding real-time audio samples.
//!     buffer: Audio<Mono32>,
//! }
//!
//! impl App<'_> {
//!     /// Speaker is ready to play more audio.
//!     fn play(&mut self, mut sink: SpeakersSink<Mono32>) -> Poll<()> {
//!         sink.stream(self.buffer.drain());
//!         Pending
//!     }
//!
//!     /// Microphone has recorded some audio.
//!     fn record(&mut self, stream: MicrophoneStream<Mono32>) -> Poll<()> {
//!         self.buffer.extend(stream);
//!         Pending
//!     }
//!
//!     /// Program start.
//!     async fn main(_executor: Executor) {
//!         let speakers = &mut Speakers::default();
//!         let microphone = &mut Microphone::default();
//!         let buffer = Audio::with_silence(48_000, 0);
//!         let mut app = App {
//!             speakers,
//!             microphone,
//!             buffer,
//!         };
//!
//!         Join::new(&mut app)
//!             .on(|s| s.speakers, App::play)
//!             .on(|s| s.microphone, App::record)
//!             .await
//!     }
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

mod consts;
mod microphone;
mod speakers;

pub use microphone::{Microphone, MicrophoneStream};
pub use speakers::{Speakers, SpeakersSink};
