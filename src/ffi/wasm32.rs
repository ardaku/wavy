// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(unsafe_code)]

use cala_core::os::web::{JsVar, JsFn};
use crate::frame::Frame;
use std::marker::PhantomData;
use std::task::Context;
use std::task::Poll;
use std::convert::TryInto;

pub(crate) struct Player<F: Frame> {
    // JavaScript AudioContext
    audio_ctx: JsVar,
    // Sample rate of the speakers
    sample_rate: u32,
    // JavaScript audio double buffer
    buffers: JsVar,
    // If buffer_b is the one to write to next.
    is_buffer_b: bool,
    // Get the left channel ArrayBuffer
    fn_left: JsFn,
    // Get the right channel ArrayBuffer
    fn_right: JsFn,
    // Temporary buffer for audio to copy over to JavaScript all at once.
    tmp_audio_l: [f64; 1024],
    // Temporary buffer for audio to copy over to JavaScript all at once.
    tmp_audio_r: [f64; 1024],
    // JavaScript Promise
    promise: JsVar,
    // JavaScript Function for resetting promise
    set_resolve: JsVar,
    // JavaScript Function to make a new audio player Promise
    new_promise: JsFn,
    _phantom: PhantomData<F>,
}

impl<F: Frame> Player<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let _ = sr;

        let audio_ctx = unsafe {
            JsFn::new("return new (window.AudioContext || window.webkitAudioContext)();").call(None, None).unwrap()
        };
        let sample_rate: u32 = unsafe {
            JsFn::new("return param_a.sampleRate;").call(Some(&audio_ctx), None).unwrap().into_i32().try_into().unwrap()
        };
        let buffers_constructor = unsafe {
            JsFn::new("return [param_a.createBuffer(2, 1024, param_a.sampleRate), param_a.createBuffer(2, 1024, param_a.sampleRate)];")
        };
        let buffers = unsafe {
            buffers_constructor.call(Some(&audio_ctx), None).unwrap()
        };
        let is_buffer_b = false;
        let fn_left = unsafe { JsFn::new("return param_a[param_b].getChannelData(0);") };
        let fn_right = unsafe { JsFn::new("return param_a[param_b].getChannelData(1);") };
        let tmp_audio_l = [0.0f64; 1024];
        let tmp_audio_r = [0.0f64; 1024];
        // param_a: AudioContext, param_b: [AudioBuffer; 2]
        let audio_callback = unsafe { JsFn::new("\
            return function(next, buffer_idx, resolve) {\
                resolve();
                var size = 1024.0 / param_a.sampleRate;\
                next += size * 2.0;\
                var a_source = param_a.createBufferSource();\
                a_source.onended = play_a(next, buffer_idx, resolve);\
                a_source.buffer = param_b[buffer_idx];\
                a_source.connect(param_a.destination);\
                a_source.start(next);\
            };\
        ").call(Some(&audio_ctx), Some(&buffers)).unwrap() };
        // param_a: AudioContext, param_b: Function (callback)
        let set_resolve = unsafe { JsFn::new("\
            var promise_res = function(value) { };\
            function resolve() { promise_res(1024); }\
            var output_buffer = audioCtx.createBuffer(2, 1024, param_a.sampleRate);\
            for (var channel = 0; channel < output_buffer.numberOfChannels; channel++) {\
                var nowBuffering = output_buffer.getChannelData(channel);\
                for (var i = 0; i < output_buffer.length; i++) {\
                    nowBuffering[i] = 0.0;\
                }\
            }\
            var size = 1024.0 / param_a.sampleRate;\
            var next_a = param_a.currentTime;\
            var next_b = next_a + size;\
            var a_source = param_a.createBufferSource();\
            a_source.onended = param_b(next_a, 0, resolve);\
            a_source.buffer = output_buffer;\
            a_source.connect(param_a.destination);\
            a_source.start(next_a);\
            var b_source = param_a.createBufferSource();\
            b_source.onended = param_b(next_b, 1, resolve);\
            b_source.buffer = output_buffer;\
            b_source.connect(param_a.destination);\
            b_source.start(next_b);\
            return function(presolve) {
                promise_res = presolve;
            }\
        ").call(Some(&audio_ctx), Some(&audio_callback)).unwrap() };
        let new_promise = unsafe { JsFn::new("\
            return new Promise(function(res, rej) {\
                param_a(res);\
            });\
        ") };
        let promise = unsafe { new_promise.call(Some(&set_resolve), None).unwrap() };

        Some(Self { promise,new_promise,set_resolve, tmp_audio_l,tmp_audio_r,audio_ctx, sample_rate, buffers,is_buffer_b,fn_left,fn_right,_phantom })
    }

    pub(crate) fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(_result) = self.promise.poll() {
            self.promise = unsafe { self.new_promise.call(Some(&self.set_resolve), None).unwrap() };
            Poll::Ready(())
        } else {
            unsafe { self.promise.set_waker() };
            Poll::Pending
        }
    }

    pub(crate) fn play_last(&mut self, audio: &[F]) -> usize {
        let _ = audio;
        
        let buffer = JsVar::from_i32(if self.is_buffer_b {
            1
        } else {
            0
        });
        self.is_buffer_b = !self.is_buffer_b;
        unsafe {
            let buffer_l = self.fn_left.call(Some(&self.buffers), Some(&buffer)).unwrap();
            let buffer_r = self.fn_right.call(Some(&self.buffers), Some(&buffer)).unwrap();
            buffer_l.write_doubles(&self.tmp_audio_l);
            buffer_r.write_doubles(&self.tmp_audio_r);
        }

        0 // 0 frames were written.
    }
}

pub(crate) struct Recorder<F: Frame> {
    _phantom: PhantomData<F>,
}

impl<F: Frame> Recorder<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let _ = sr;

        None
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let _ = cx;
    
        Poll::Pending
    }

    pub(crate) fn record_last(&mut self, audio: &mut Vec<F>) {
        let _ = audio;
    }
}
