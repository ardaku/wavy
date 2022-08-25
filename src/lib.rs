// Copyright © 2019-2021 The Wavy Contributors.
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
//! FIXME: Update example.
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

// mod consts;
// mod env;
// mod listener;
// mod microphone;
// mod speakers;

// pub use listener::Listener;
// pub use microphone::{Microphone, Recorder};
// pub use speakers::{Player, Speakers};

pub use connector::Connector;
pub use microphone::Microphone;
pub use speakers::Speakers;

mod speakers {
    use std::fmt::{Debug, Display, Formatter, Result};
    use crate::platform::{PlatformSpeakers, Platform, Support};

    /// Speakers future - plays audio.
    pub struct Speakers(pub(crate) PlatformSpeakers);

    impl Display for Speakers {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            <Self as Debug>::fmt(self, f)
        }
    }

    impl Debug for Speakers {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", Platform.speakers_name(&self.0))
        }
    }
}

mod microphone {
    use std::fmt::{Debug, Display, Formatter, Result};
    use crate::platform::{PlatformMicrophone, Platform, Support};

    /// Microphone future - records audio.
    pub struct Microphone(pub(crate) PlatformMicrophone);

    impl Display for Microphone {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            <Self as Debug>::fmt(self, f)
        }
    }

    impl Debug for Microphone {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", Platform.microphone_name(&self.0))
        }
    }
}

mod connector {
    use std::fmt::{Debug, Formatter, Result};
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use crate::platform::{Platform, Support};
    use crate::Microphone;
    use crate::Speakers;

    pub trait Connectable {
        fn connect() -> Pin<Box<dyn Future<Output = Self>>>;
    }

    impl Connectable for Microphone {
        fn connect() -> Pin<Box<dyn Future<Output = Self>>> {
            Box::pin(Platform.query_microphones())
        }
    }

    impl Connectable for Speakers {
        fn connect() -> Pin<Box<dyn Future<Output = Self>>> {
            Box::pin(Platform.query_speakers())
        }
    }

    /// Connector for speakers and microphones.
    pub struct Connector<C: Connectable>(Pin<Box<dyn Future<Output = C>>>);

    impl Debug for Connector<Microphone> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "MicrophoneConnector")
        }
    }
    
    impl Debug for Connector<Speakers> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "SpeakersConnector")
        }
    }

    impl<C: Connectable> Connector<C> {
        /// Create a new future to connect to either new speakers or new
        /// microphones.
        pub fn new() -> Self {
            Self(C::connect())
        }
    }

    impl<C: Connectable> Future for Connector<C> {
        type Output = C;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<C> {
            Pin::new(&mut self.get_mut().0).poll(cx)
        }
    }

    impl<C: Connectable> Default for Connector<C> {
        fn default() -> Self {
            Self::new()
        }
    }
}

mod platform {
    #[path = "../linux/linux.rs"]
    mod linux;

    pub(crate) use linux::{
        Platform, PlatformMicrophone, PlatformMicrophoneQuery,
        PlatformSpeakersQuery, PlatformSpeakers,
    };

    pub(crate) trait Support {
        fn query_speakers(self) -> PlatformSpeakersQuery;
        fn query_microphones(self) -> PlatformMicrophoneQuery;
        fn speakers_name(self, speakers: &PlatformSpeakers) -> &str;
        fn microphone_name(self, microphone: &PlatformMicrophone) -> &str;
    }
}
