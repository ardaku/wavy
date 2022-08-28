// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(clippy::needless_doctest_main)]

use std::fmt::{Debug, Display, Formatter, Result};

use fon::{chan::Ch32, Frame, Resampler, Sink};
use pasts::prelude::*;

use crate::ffi;

/// Play audio through speakers.  Notifier produces an audio sink, which
/// consumes an audio stream of played samples.  If you don't write to the sink,
/// it will keep playing whatever was last streamed into it.
///
/// # 440 HZ Sine Wave Example
/// **note:** This example depends on `twang = "0.5"` to synthesize the sine
/// wave.
/// ```
/// use fon::{stereo::Stereo32, Sink};
/// use pasts::{prelude::*, Join};
/// use twang::{Fc, Signal, Synth};
/// use wavy::{Speakers, SpeakersSink};
///
/// /// Shared state between tasks on the thread.
/// struct App<'a> {
///     /// Handle to stereo speakers
///     speakers: &'a mut Speakers<2>,
///     /// A streaming synthesizer using Twang.
///     synth: Synth<()>,
/// }
///
/// impl App<'_> {
///     /// Speaker is ready to play more audio.
///     fn play(&mut self, mut sink: SpeakersSink<Stereo32>) -> Poll<()> {
///         sink.stream(&mut self.synth);
///         Pending
///     }
///
///     /// Program start.
///     async fn main(_executor: Executor) {
///         fn sine(_: &mut (), fc: Fc) -> Signal {
///             fc.freq(440.0).sine().gain(0.7)
///         }
///
///         let speakers = &mut Speakers::default();
///         let synth = Synth::new((), sine);
///         let mut app = App { speakers, synth };
///
///         Join::new(&mut app).on(|s| s.speakers, App::play).await;
///     }
/// }
/// ```
#[derive(Default)]
pub struct Speakers<const N: usize>(pub(super) ffi::Speakers);

impl<const N: usize> Display for Speakers<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

impl<const N: usize> Debug for Speakers<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Speakers<0> {
    /// Query available audio destinations.
    pub fn query() -> Vec<Self> {
        ffi::device_list(Self)
    }
}

impl<const N: usize> Speakers<N> {
    /// Try a reconfiguration of speakers.
    pub fn config<const C: usize>(
        self,
    ) -> std::result::Result<Speakers<C>, Self>
    where
        Speakers<C>: SpeakersProperties,
    {
        let bit = C - 1;
        if (self.0.channels() & (1 << bit)) != 0 {
            Ok(Speakers(self.0))
        } else {
            Err(self)
        }
    }
}

pub trait SpeakersProperties {
    type Sample: Frame<Chan = Ch32>;
}

impl SpeakersProperties for Speakers<1> {
    type Sample = fon::mono::Mono32;
}

impl SpeakersProperties for Speakers<2> {
    type Sample = fon::stereo::Stereo32;
}

impl SpeakersProperties for Speakers<6> {
    type Sample = fon::surround::Surround32;
}

impl<const N: usize> Notifier for Speakers<N>
where
    Speakers<N>: SpeakersProperties,
{
    type Event = SpeakersSink<<Self as SpeakersProperties>::Sample>;

    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<Self::Event> {
        let this = self.get_mut();
        if let Ready(()) = Pin::new(&mut this.0).poll(e) {
            Ready(SpeakersSink(this.0.play()))
        } else {
            Pending
        }
    }
}

/// A sink that consumes audio samples and plays them through the speakers.
pub struct SpeakersSink<F: Frame<Chan = Ch32>>(ffi::SpeakersSink<F>);

impl<F: Frame<Chan = Ch32>> Debug for SpeakersSink<F> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "SpeakersSink(rate: {})", self.sample_rate())
    }
}

impl<F: Frame<Chan = Ch32>> Sink<F> for SpeakersSink<F> {
    fn sample_rate(&self) -> f64 {
        self.0.sample_rate()
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        self.0.resampler()
    }

    fn buffer(&mut self) -> &mut [F] {
        self.0.buffer()
    }
}
