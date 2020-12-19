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

use fon::{Sample, stereo::Stereo32, Audio};

pub(crate) struct Speakers<S: Sample> {
    _phantom: PhantomData<S>,
}

impl<S: Sample> Speakers<S> {
    pub(crate) fn connect(_id: &crate::SpeakerId) -> Option<(Self, u32)> {
        let state = super::state();
        let _phantom = PhantomData::<S>;

        // Lazily Initialize audio context & processor node.
        state.lazy_init();

        // Check if already connected
        if state.speaker.is_some() {
            panic!("Already connected to speakers!");
        }

        // Initialize speakers.
        state.speaker = Some(state.context.as_mut().unwrap().destination());

        // Connect speakers. FIXME
        state
            .proc
            .as_ref()
            .unwrap()
            .connect_with_audio_node(state.speaker.as_ref().unwrap()).ok()?;

        Some((Self { _phantom }, super::SAMPLE_RATE))
    }

    pub(crate) fn play(&mut self, audio: &Audio<S>) -> usize {
        // Grab global state.
        let state = super::state();

        // Convert to speaker's native type.
        for (i, sample) in audio.iter().enumerate() {
            if i == super::PERIOD.into() {
                break;
            }
            let sample: Stereo32 = sample.convert();
            state.l_buffer[i] = sample.channels()[0].into();
            state.r_buffer[i] = sample.channels()[1].into();
        }

        // Always writes the same number of samples.
        super::PERIOD.into()
    }
}

impl<S: Sample> Future for &mut Speakers<S> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = super::state();
        if state.played {
            state.played = false;
            Poll::Ready(())
        } else {
            state.speaker_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
