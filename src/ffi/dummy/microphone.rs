// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{
    chan::{Ch16, Channel},
    sample::Sample1,
    Resampler, Stream,
};

pub(crate) struct Microphone<C: Channel + Unpin> {
    stream: MicrophoneStream<C>,
}

impl<C: Channel + Unpin> Microphone<C> {
    pub(crate) fn new() -> Option<Self> {
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
    resampler: Resampler<Sample1<C>>,
}

impl<C> Stream<Sample1<C>> for &mut MicrophoneStream<C>
where
    C: Channel + Unpin + From<Ch16>,
{
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn stream_sample(&mut self) -> Option<Sample1<C>> {
        None
    }

    fn resampler(&mut self) -> &mut Resampler<Sample1<C>> {
        &mut self.resampler
    }
}
