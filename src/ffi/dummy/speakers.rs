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
