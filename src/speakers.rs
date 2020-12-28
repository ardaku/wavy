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
/// use std::cell::RefCell;
///
/// use fon::mono::Mono64;
/// use pasts::prelude::*;
/// use twang::Synth;
/// use wavy::SpeakerId;
///
/// /// The program's shared state.
/// struct State {}
///
/// /// Speakers task (play sine wave).
/// async fn speakers(state: &RefCell<State>) {
///     // Connect to system's speaker(s)
///     let mut speakers = SpeakerId::default().connect::<Mono64>().unwrap();
///     // Create a new synthesizer
///     let mut synth = Synth::new();
///
///     loop {
///         // 1. Wait for speaker to need more samples.
///         let audio = speakers.play().await;
///         // 2. Borrow shared state mutably
///         let _state = state.borrow_mut();
///         // 3. Generate and write samples into speaker buffer.
///         synth.gen(audio, |fc| fc.freq(440.0).sine().gain(0.7));
///     }
/// }
///
/// /// Program start.
/// async fn start() {
///     // Initialize shared state.
///     let state = RefCell::new(State {});
///     // Create and wait on speaker task.
///     speakers(&state).await;
/// }
///
/// /// Start the async executor.
/// fn main() {
///     exec!(start());
/// }
/// ```
pub struct Speakers(pub(super) ffi::Speakers);

impl Debug for Speakers {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(
            fmt,
            "Speakers(rate: {:?}, channels: {})",
            self.0.sample_rate,
            self.0.channels
        )
    }
}

impl Speakers {
    /// Check is speakers are available to use in a specific configuration
    pub fn avail<F>(&mut self) -> bool
    where
        F: Frame<Chan = Ch32>,
    {
        self.0.set_channels::<F>().is_some()
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
