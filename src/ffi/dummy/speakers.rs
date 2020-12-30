// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::{
    fmt::{Display, Error, Formatter},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{chan::Ch32, Frame, Resampler, Sink};

use super::SoundDevice;

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
