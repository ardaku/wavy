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
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{chan::Ch16, sample::Sample, Audio};

pub(crate) struct Speakers<S: Sample> {
    _phantom: PhantomData<S>,
}

impl<S: Sample> Speakers<S> {
    pub(crate) fn connect(_id: &crate::SpeakerId) -> Option<(Self, u32)> {
        let _phantom = PhantomData::<S>;

        Some((Self { _phantom }, 48_000))
    }

    pub(crate) fn play(&mut self, audio: &Audio<S>) -> usize {
        let _ = audio;

        0 // 0 frames were written.
    }
}

impl<S: Sample + Unpin> Future for &mut Speakers<S>
where
    Ch16: From<S::Chan>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
