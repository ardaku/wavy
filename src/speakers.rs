// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(clippy::needless_doctest_main)]

use std::fmt::{Debug, Display, Formatter, Result};

use fon::{chan::Ch32, Frame, Resampler, Sink};

use crate::ffi;

/// Play audio samples through a speaker.
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
#[derive(Default)]
pub struct Speakers(pub(super) ffi::Speakers);

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

impl Speakers {
    /// Query available audio destinations.
    pub fn query() -> Vec<Self> {
        ffi::device_list(Self)
    }

    /// Check is speakers are available to use in a specific configuration
    pub fn supports<F>(&self) -> bool
    where
        F: Frame<Chan = Ch32>,
    {
        let count = F::CHAN_COUNT;
        let bit = count - 1;
        (self.0.channels() & (1 << bit)) != 0
    }

    /// Play audio through speakers.  Returns an audio sink, which consumes an
    /// audio stream of played samples.  If you don't write to the sink, it will
    /// keep playing whatever was last streamed into it.
    pub async fn play<F: Frame<Chan = Ch32>>(&mut self) -> SpeakersSink<'_, F> {
        (&mut self.0).await;
        SpeakersSink(self.0.play())
    }
}

/// A sink that consumes audio samples and plays them through the speakers.
pub struct SpeakersSink<'a, F: Frame<Chan = Ch32>>(ffi::SpeakersSink<'a, F>);

impl<F: Frame<Chan = Ch32>> Debug for SpeakersSink<'_, F> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "SpeakersSink(rate: {})", self.sample_rate())
    }
}

impl<F: Frame<Chan = Ch32>> Sink<F> for SpeakersSink<'_, F> {
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
