// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::{
    fmt::{Display, Error, Formatter},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{chan::Ch32, Frame, Resampler, Sink};

use super::{AudioDevice ,SoundDevice};

pub(crate) struct Speakers {
    pub(crate) sample_rate: Option<f64>,
}

impl SoundDevice for Speakers {
    const INPUT: bool = false;
}

impl Display for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Default")
    }
}

impl Default for Speakers {
    fn default() -> Self {
        Speakers { sample_rate: Some(48_000.0) }
    }
}

impl From<AudioDevice> for Speakers {
    fn from(this: AudioDevice) -> Self {
        Self::default()
    }
}

impl Speakers {
    pub(crate) fn play<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> SpeakersSink<'_, F> {
        SpeakersSink(self, Resampler::default(), PhantomData)
    }

    pub(crate) fn channels(&self) -> u8 {
        1
    }
}

impl Future for &mut Speakers {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub(crate) struct SpeakersSink<'a, F: Frame<Chan = Ch32>>(
    &'a mut Speakers,
    Resampler<F>,
    PhantomData<F>,
);

impl<F: Frame<Chan = Ch32>> Sink<F> for SpeakersSink<'_, F> {
    fn sample_rate(&self) -> f64 {
        self.0.sample_rate.unwrap()
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.1
    }

    fn buffer(&mut self) -> &mut [F] {
        &mut []
    }
}
