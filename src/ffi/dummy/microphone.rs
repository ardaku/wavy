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
    pin::Pin,
    task::{Context, Poll},
};

use fon::{
    chan::{Ch16, Channel},
    mono::Mono,
    Resampler, Stream,
};

pub(crate) struct Microphone<C: Channel + Unpin> {
    stream: MicrophoneStream<C>,
}

impl<C: Channel + Unpin> Microphone<C> {
    pub(crate) fn new(_id: &crate::MicrophoneId) -> Option<Self> {
        None
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.stream.sample_rate
    }

    pub(crate) fn record(&mut self) -> &mut MicrophoneStream<C> {
        &mut self.stream
    }
}

impl<C: Channel + Unpin> Future for Microphone<C> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub(crate) struct MicrophoneStream<C: Channel + Unpin> {
    // Sample rate of the stream.
    sample_rate: u32,
    // Stream's resampler
    resampler: Resampler<Mono<C>>,
}

impl<C> Stream<Mono<C>> for &mut MicrophoneStream<C>
where
    C: Channel + Unpin + From<Ch16>,
{
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn stream_sample(&mut self) -> Option<Mono<C>> {
        None
    }

    fn resampler(&mut self) -> &mut Resampler<Mono<C>> {
        &mut self.resampler
    }
}
