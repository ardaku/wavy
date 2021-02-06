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

use fon::{chan::Ch32, Frame, Stream};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    MediaStream, MediaStreamAudioSourceNode, MediaStreamAudioSourceOptions,
    MediaStreamConstraints,
};

use super::SoundDevice;

pub(crate) struct Microphone();

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Default")
    }
}

impl SoundDevice for Microphone {
    const INPUT: bool = true;
}

impl Default for Microphone {
    fn default() -> Self {
        let state = super::state();

        // Lazily Initialize audio context & processor node.
        state.lazy_init();

        // Prompt User To Connect Microphone.
        let md = web_sys::window()
            .unwrap()
            .navigator()
            .media_devices()
            .ok()
            .unwrap();
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

        Self()
    }
}

impl Microphone {
    pub(crate) fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<'_, F> {
        MicrophoneStream {
            index: 0,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn channels(&self) -> u8 {
        0b0000_0001
    }
}

impl Future for Microphone {
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

pub(crate) struct MicrophoneStream<'a, F: Frame<Chan = Ch32>> {
    // Index into buffer
    index: usize,
    //
    _phantom: PhantomData<&'a F>,
}

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        // Grab global state.
        let state = super::state();

        if self.index == state.i_buffer.len() {
            return None;
        }
        let frame = F::from_channels(&[Ch32::new(state.i_buffer[self.index])]);
        self.index += 1;
        Some(frame)
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<'_, F> {
    fn sample_rate(&self) -> Option<f64> {
        Some(super::state().sample_rate.unwrap())
    }

    fn len(&self) -> Option<usize> {
        Some(super::BUFFER_SIZE.into())
    }
}
