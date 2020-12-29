// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::fmt::{Debug, Formatter, Result};

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
/// use wavy::{SpeakersId, SpeakersSink};
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
///     fn event(&mut self, event: Event<'_>) -> bool {
///         match event {
///             Event::Play(mut speakers) => speakers.stream(&mut self.synth),
///         }
///         true
///     }
/// }
/// 
/// /// Program start.
/// fn main() {
///     fn sine(_: (), fc: Fc) -> Signal {
///         fc.freq(440.0).sine().gain(0.7)
///     }
/// 
///     let mut state = State { synth: Synth::new((), sine) };
///     let mut speakers = SpeakersId::default().connect().unwrap();
/// 
///     exec! { state.event( wait! [
///         Event::Play(speakers.play().await),
///     ] .await ) }
/// }
/// ```
pub struct Speakers(pub(super) ffi::Speakers);

impl Debug for Speakers {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(
            fmt,
            "Speakers(rate: {:?}, channels: {})",
            self.0.sample_rate, self.0.channels
        )
    }
}

impl Speakers {
    /// Check is speakers are available to use in a specific configuration
    pub fn avail<F>(&mut self) -> bool
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
