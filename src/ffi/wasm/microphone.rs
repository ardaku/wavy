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

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    MediaStream, MediaStreamAudioSourceNode, MediaStreamAudioSourceOptions,
    MediaStreamConstraints,
};

use fon::{chan::Channel, sample::Sample1, Resampler, Stream};

pub(crate) struct Microphone<C: Channel + Unpin> {
    stream: MicrophoneStream<C>,
}

impl<C: Channel + Unpin> Microphone<C> {
    pub(crate) fn new(_id: &crate::MicrophoneId) -> Option<Self> {
        let state = super::state();

        // Lazily Initialize audio context & processor node.
        state.lazy_init();

        // Prompt User To Connect Microphone.
        let md = web_sys::window()
            .unwrap()
            .navigator()
            .media_devices()
            .ok()?;
        let promise = md
            .get_user_media_with_constraints(
                MediaStreamConstraints::new().audio(&JsValue::TRUE),
            )
            .unwrap();
        #[allow(trivial_casts)] // Actually needed here.
        let cb = Closure::wrap(Box::new(|media_stream| {
            let state = super::state();
            // Create audio source from media stream.
            let audio_src = MediaStreamAudioSourceNode::new(
                state.context.as_ref().unwrap(),
                &MediaStreamAudioSourceOptions::new(
                    &MediaStream::unchecked_from_js(media_stream),
                ),
            )
            .unwrap();

            // Connect microphones to processor node.
            audio_src
                .connect_with_audio_node(state.proc.as_ref().unwrap())
                .unwrap();

            // Add to connected microphones (refresh browser to remove).
            state.microphone.push(audio_src);
        }) as Box<dyn FnMut(_)>);
        let _ = promise.then(&cb);
        cb.forget();

        Some(Self {
            stream: MicrophoneStream {
                audio: vec![0.0; super::PERIOD.into()],
                index: super::PERIOD.into(),
                resampler: Resampler::new(),
            },
        })
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        super::SAMPLE_RATE
    }

    pub(crate) fn record(&mut self) -> &mut MicrophoneStream<C> {
        // Grab global state.
        let state = super::state();

        // Convert to requested audio type.
        for (i, sample) in state.i_buffer.iter().enumerate() {
            if i == super::PERIOD.into() {
                break;
            }
            self.stream.audio[i] = *sample;
        }

        self.stream.index = 0;

        &mut self.stream
    }
}

impl<C: Channel + Unpin> Future for Microphone<C> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = super::state();
        if state.recorded {
            state.recorded = false;
            Poll::Ready(())
        } else {
            state.mics_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub(crate) struct MicrophoneStream<C: Channel + Unpin> {
    // Stream's resampler
    resampler: Resampler<Sample1<C>>,
    // Buffer
    audio: Vec<f32>,
    // Index into buffer
    index: usize,
}

impl<C> Stream<Sample1<C>> for &mut MicrophoneStream<C>
where
    C: Channel + Unpin,
{
    fn sample_rate(&self) -> u32 {
        super::SAMPLE_RATE
    }

    fn stream_sample(&mut self) -> Option<Sample1<C>> {
        if self.index == self.audio.len() {
            return None;
        }
        let sample: C = C::from(self.audio[self.index].into());
        self.index += 1;
        Some(Sample1::new(sample))
    }

    fn resampler(&mut self) -> &mut Resampler<Sample1<C>> {
        &mut self.resampler
    }
}
