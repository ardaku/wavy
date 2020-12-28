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

use fon::{chan::Ch32, Frame, Stream};

use crate::ffi;

/// Record audio samples from a microphone.
pub struct Microphone(pub(super) ffi::Microphone);

impl Debug for Microphone {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(
            fmt,
            "Microphone(rate: {:?}, channels: {})",
            self.0.sample_rate,
            self.0.channels
        )
    }
}

impl Microphone {
    /// Check is speakers are available to use in a specific configuration
    pub fn avail<F>(&mut self) -> bool
    where
        F: Frame<Chan = Ch32>,
    {
        self.0.set_channels::<F>().is_some()
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
