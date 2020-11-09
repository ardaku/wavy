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

use fon::{chan::Ch32, sample::Sample, stereo::Stereo32, Audio};

pub(crate) struct Speakers<S: Sample>
where
    Ch32: From<S::Chan>,
{
    _phantom: PhantomData<S>,
}

impl<S: Sample> Speakers<S>
where
    Ch32: From<S::Chan>,
{
    pub(crate) fn connect() -> (Self, u32) {
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

        // Connect speakers.
        state.proc.as_ref().unwrap().connect_with_audio_node(state.speaker.as_ref().unwrap()).unwrap();

        (Self { _phantom }, super::SAMPLE_RATE)
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

impl<S: Sample + Unpin> Future for &mut Speakers<S>
where
    Ch32: From<S::Chan>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = super::state();
        if state.played {
            state.played = false;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
