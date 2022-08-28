// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::fmt::{Debug, Display, Formatter, Result};

use fon::{chan::Ch32, Frame, Stream};
use pasts::prelude::*;

use crate::ffi;

/// Record audio from connected microphone.  Notifier produces an audio stream,
/// which contains the samples recorded since the previous call.
#[derive(Default)]
pub struct Microphone<const N: usize>(pub(super) ffi::Microphone);

impl<const N: usize> Display for Microphone<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

impl<const N: usize> Debug for Microphone<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Microphone<0> {
    /// Query available audio sources.
    pub fn query() -> Vec<Self> {
        ffi::device_list(Self)
    }
}

impl<const N: usize> Microphone<N> {
    /// Try a reconfiguration of microphone.
    pub fn config<const C: usize>(
        self,
    ) -> std::result::Result<Microphone<C>, Self>
    where
        Microphone<C>: MicrophoneProperties,
    {
        let bit = C - 1;
        if (self.0.channels() & (1 << bit)) != 0 {
            Ok(Microphone(self.0))
        } else {
            Err(self)
        }
    }
}

pub trait MicrophoneProperties {
    type Sample: Frame<Chan = Ch32>;
}

impl MicrophoneProperties for Microphone<1> {
    type Sample = fon::mono::Mono32;
}

impl MicrophoneProperties for Microphone<2> {
    type Sample = fon::stereo::Stereo32;
}

impl MicrophoneProperties for Microphone<6> {
    type Sample = fon::surround::Surround32;
}

impl<const N: usize> Notifier for Microphone<N>
where
    Microphone<N>: MicrophoneProperties,
{
    type Event = MicrophoneStream<<Self as MicrophoneProperties>::Sample>;

    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<Self::Event> {
        let this = self.get_mut();
        if let Ready(()) = Pin::new(&mut this.0).poll(e) {
            Ready(MicrophoneStream(this.0.record()))
        } else {
            Pending
        }
    }
}

/// A stream of recorded audio samples from a microphone.
pub struct MicrophoneStream<F: Frame<Chan = Ch32>>(ffi::MicrophoneStream<F>);

impl<F: Frame<Chan = Ch32>> Debug for MicrophoneStream<F> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "MicrophoneStream(rate: {:?})", self.sample_rate())
    }
}

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<F> {
    fn sample_rate(&self) -> Option<f64> {
        self.0.sample_rate()
    }

    fn len(&self) -> Option<usize> {
        self.0.len()
    }
}
