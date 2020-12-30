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
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{chan::Ch32, chan::Channel, mono::Mono, Frame, Resampler, Stream};

pub(crate) struct Microphone {
    pub(crate) channels: u8,
    pub(crate) sample_rate: f64,
}

impl Microphone {
    pub(crate) fn new(_id: crate::MicrophoneId) -> Option<Self> {
        None
    }

    pub(crate) fn sample_rate(&self) -> f64 {
        crate::consts::SAMPLE_RATE.into()
    }

    pub(crate) fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<'_, F> {
        MicrophoneStream(PhantomData)
    }

    pub(crate) fn channels(&self) -> u8 {
        1
    }
}

impl Future for Microphone {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub(crate) struct MicrophoneStream<'a, F: Frame<Chan = Ch32>>(
    PhantomData<&'a F>,
);

impl<F: Frame<Chan = Ch32>> Iterator for &mut MicrophoneStream<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<F> {
        None
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for &mut MicrophoneStream<'_, F> {
    fn sample_rate(&self) -> Option<f64> {
        Some(crate::consts::SAMPLE_RATE.into())
    }

    fn len(&self) -> Option<usize> {
        Some(0)
    }
}
