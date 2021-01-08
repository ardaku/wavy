// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::fmt::{Debug, Display, Formatter, Result};

use fon::{chan::Ch32, Frame, Stream};

use crate::ffi;

/// Record audio samples from a microphone.
#[derive(Default)]
pub struct Microphone(pub(super) ffi::Microphone);

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

impl Debug for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Microphone {
    /// Query available audio sources.
    pub fn query() -> Vec<Self> {
        ffi::device_list(Self)
    }

    /// Check is microphone is available to use in a specific configuration
    pub fn supports<F>(&self) -> bool
    where
        F: Frame<Chan = Ch32>,
    {
        let count = F::CHAN_COUNT;
        let bit = count - 1;
        (self.0.channels() & (1 << bit)) != 0
    }

    /// Record audio from connected microphone.  Returns an audio stream, which
    /// contains the samples recorded since the previous call.
    pub async fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<'_, F> {
        (&mut self.0).await;
        MicrophoneStream(self.0.record())
    }
}

/// A stream of recorded audio samples from a microphone.
pub struct MicrophoneStream<'a, F: Frame<Chan = Ch32>>(
    ffi::MicrophoneStream<'a, F>,
);

impl<F: Frame<Chan = Ch32>> Debug for MicrophoneStream<'_, F> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "MicrophoneStream(rate: {:?})", self.sample_rate())
    }
}

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<'_, F> {
    fn sample_rate(&self) -> Option<f64> {
        self.0.sample_rate()
    }

    fn len(&self) -> Option<usize> {
        self.0.len()
    }
}
