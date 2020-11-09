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

use web_sys::{MediaStream, MediaStreamAudioSourceNode, MediaStreamAudioSourceOptions, MediaStreamConstraints};
use wasm_bindgen::{JsValue, closure::Closure, JsCast};

use fon::{
    chan::{Channel},
    sample::Sample1,
    Resampler, Stream,
};

pub(crate) struct Microphone<C: Channel + Unpin> {
    stream: MicrophoneStream<C>,
}

impl<C: Channel + Unpin> Microphone<C> {
    pub(crate) fn new() -> Option<Self> {
        let state = super::state();
    
        // Lazily Initialize audio context & processor node.
        state.lazy_init();
        
        // Prompt User To Connect Microphone.
        let md = web_sys::window().unwrap().navigator().media_devices().ok()?;
        let promise = md.get_user_media_with_constraints(
            MediaStreamConstraints::new().audio(&JsValue::TRUE)
        ).unwrap();
        #[allow(trivial_casts)] // Actually needed here.
        let cb = Closure::wrap(Box::new(|media_stream| {
            let state = super::state();
        
            state.microphone.push(MediaStreamAudioSourceNode::new(
                state.context.as_ref().unwrap(),
                &MediaStreamAudioSourceOptions::new(&MediaStream::unchecked_from_js(media_stream)),
            ).unwrap());
            state.microphone_waker.push(None);
            
            // FIXME
        }) as Box<dyn FnMut(_)>);
        let _ = promise.then(&cb);

        Some(Self { stream: MicrophoneStream { resampler: Resampler::new(),
             } })
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        super::SAMPLE_RATE
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
    // Stream's resampler
    resampler: Resampler<Sample1<C>>,
}

impl<C> Stream<Sample1<C>> for &mut MicrophoneStream<C>
where
    C: Channel + Unpin,
{
    fn sample_rate(&self) -> u32 {
        super::SAMPLE_RATE
    }

    fn stream_sample(&mut self) -> Option<Sample1<C>> {
        None
    }

    fn resampler(&mut self) -> &mut Resampler<Sample1<C>> {
        &mut self.resampler
    }
}
