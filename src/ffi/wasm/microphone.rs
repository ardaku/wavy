// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::{
    fmt::{Display, Error, Formatter},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    task::{Context, Poll},
};

use fon::{chan::Ch32, Frame, Stream};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    MediaStream, MediaStreamAudioSourceNode, MediaStreamAudioSourceOptions,
    MediaStreamConstraints,
};

use super::SoundDevice;

pub(crate) struct Microphone(*mut AtomicBool);

#[allow(unsafe_code)]
impl Drop for Microphone {
    fn drop(&mut self) {
        // Safety
        if unsafe { (*self.0).load(SeqCst) } {
            eprintln!("Microphone dropped before dropping stream");
            std::process::exit(1);
        }

        unsafe { drop(Box::from_raw(self.0)) };
    }
}

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

        Self(Box::leak(Box::new(AtomicBool::new(false))))
    }
}

impl Microphone {
    pub(crate) fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<F> {
        MicrophoneStream {
            microphone: self.0,
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

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety
        if unsafe { (*self.0).load(SeqCst) } {
            eprintln!("Tried to poll microphone before dropping stream");
            std::process::exit(1);
        }
        let inner = unsafe { self.0.as_mut().unwrap() };

        let state = super::state();
        if state.recorded {
            state.recorded = false;
            inner.store(true, SeqCst);
            Poll::Ready(())
        } else {
            state.mics_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub(crate) struct MicrophoneStream<F: Frame<Chan = Ch32>> {
    //
    microphone: *mut AtomicBool,
    // Index into buffer
    index: usize,
    //
    _phantom: PhantomData<&'static F>,
}

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<F> {
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

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<F> {
    fn sample_rate(&self) -> Option<f64> {
        Some(super::state().sample_rate.unwrap())
    }

    fn len(&self) -> Option<usize> {
        Some(super::BUFFER_SIZE.into())
    }
}

#[allow(unsafe_code)]
impl<F: Frame<Chan = Ch32>> Drop for MicrophoneStream<F> {
    fn drop(&mut self) {
        let mic = unsafe { self.microphone.as_mut().unwrap() };
        // Unlock
        mic.store(false, SeqCst);
    }
}
