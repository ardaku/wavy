// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(clippy::needless_doctest_main)]

use crate::env::Playback;
use flume::Receiver;
use fon::{chan::Ch32, Frame, Sink};
use std::fmt::{Debug, Display, Formatter, Result};
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Play audio samples through a speaker.
///
/// Speakers are always implemented with support for 7.1 surround sound.  The
/// channel order is compatible with 5.1 surround and 2.0 stereo.  If the host
/// environment needs to convert, the audio will be mixed down in a way that
/// doesn't reduce amplitude.
///
/// # 440 HZ Sine Wave Example
/// **note:** This example depends on `twang = "0.5"` to synthesize the sine
/// wave.
/// ```no_run
/// use fon::{stereo::Stereo32, Sink};
/// use pasts::{exec, wait};
/// use twang::{Fc, Signal, Synth};
/// use wavy::{Speakers, SpeakersSink};
///
/// /// An event handled by the event loop.
/// enum Event<'a> {
///     /// Speaker is ready to play more audio.
///     Play(SpeakersSink<'a, Stereo32>),
/// }
///
/// /// Shared state between tasks on the thread.
/// struct State {
///     /// A streaming synthesizer using Twang.
///     synth: Synth<()>,
/// }
///
/// impl State {
///     /// Event loop.  Return false to stop program.
///     fn event(&mut self, event: Event<'_>) {
///         match event {
///             Event::Play(mut speakers) => speakers.stream(&mut self.synth),
///         }
///     }
/// }
///
/// /// Program start.
/// fn main() {
///     fn sine(_: &mut (), fc: Fc) -> Signal {
///         fc.freq(440.0).sine().gain(0.7)
///     }
///
///     let mut state = State { synth: Synth::new((), sine) };
///     let mut speakers = Speakers::default();
///
///     exec!(state.event(wait! {
///         Event::Play(speakers.play().await),
///     }));
/// }
/// ```
pub struct Speakers(Receiver<Player>);

impl Default for Speakers {
    fn default() -> Self {
        crate::env::query_speakers().recv().unwrap()
    }
}

impl Display for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

impl Debug for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<'a> Future for Speakers {
    type Output = Player;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0.recv_async())
            .poll(cx)
            .map(|x| x.unwrap())
    }
}

/// A sink that consumes audio samples and plays them through the speakers.
pub struct Player(Playback);

impl Debug for Player {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "Player(rate: {})", self.sample_rate())
    }
}

impl Sink<Ch32, 8> for Player {
    fn sample_rate(&self) -> NonZeroU32 {
        self.0.sample_rate()
    }

    fn len(&self) -> usize {
        crate::consts::CHUNK_SIZE.into()
    }

    fn sink_with<I: Iterator<Item = Frame<Ch32, 8>>>(&mut self, mut iter: I) {
        self.0.play(&mut iter);
    }
}
