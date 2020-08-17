// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(unsafe_code)]

use crate::frame::Frame;
use cala_core::os::web::{JsFn, JsPromise, JsVar};
use std::convert::TryInto;
use std::marker::PhantomData;
use std::task::Context;
use std::task::Poll;

pub(crate) struct Player<F: Frame> {
    // Sample rate of the speakers
    sample_rate: f64,
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
    promise: JsPromise<JsVar>,
    // JavaScript Function for resetting promise
    reset_promise: JsFn,
    //
    ready: bool,
    _phantom: PhantomData<F>,
}

impl<F: Frame> Player<F> {
    pub(crate) fn new() -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let audio_ctx = unsafe {
            JsFn::new(
                "return new (window.AudioContext\
                ||window.webkitAudioContext)();",
            )
            .call(None, None)
            .unwrap()
        };
        let sample_rate: f64 = unsafe {
            JsFn::new("return param_a.sampleRate;")
                .call(Some(&audio_ctx), None)
                .unwrap()
                .into_f64()
                .try_into()
                .unwrap()
        };
        let buffers_constructor = unsafe {
            JsFn::new("return [param_a.createBuffer(2, 1024, param_a.sampleRate), param_a.createBuffer(2, 1024, param_a.sampleRate)];")
        };
        let buffers = unsafe {
            buffers_constructor.call(Some(&audio_ctx), None).unwrap()
        };
        let is_buffer_b = false;
        let fn_left =
            unsafe { JsFn::new("return param_a[param_b].getChannelData(0);") };
        let fn_right =
            unsafe { JsFn::new("return param_a[param_b].getChannelData(1);") };
        let tmp_audio_l = [0.0f64; 1024];
        let tmp_audio_r = [0.0f64; 1024];
        // param_a: AudioContext, param_b: [AudioBuffer; 2]
        let audio_callback = unsafe {
            JsFn::new("\
            function play(next, buffer_idx, resolve) {\
                resolve(1024);
                var size = 1024.0 / param_a.sampleRate;\
                next += size * 2.0;\
                var a_source = param_a.createBufferSource();\
                a_source.onended = function() { play(next, buffer_idx, resolve); };\
                a_source.buffer = param_b[buffer_idx];\
                a_source.connect(param_a.destination);\
                a_source.start(next);\
            }
            return play;\
        ").call(Some(&audio_ctx), Some(&buffers)).unwrap()
        };
        // param_a: AudioContext, param_b: Function (callback)
        let reset_promise = unsafe {
            JsFn::new("\
            var resolve = function() { };\
            var output_buffer = param_a.createBuffer(2, 1024, param_a.sampleRate);\
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
            a_source.onended = param_b(next_a, 0, function(a) { resolve(a); });\
            a_source.buffer = output_buffer;\
            a_source.connect(param_a.destination);\
            a_source.start(next_a);\
            var b_source = param_a.createBufferSource();\
            b_source.onended = param_b(next_b, 1, function(a) { resolve(a); });\
            b_source.buffer = output_buffer;\
            b_source.connect(param_a.destination);\
            b_source.start(next_b);\
            return function(a, b) {\
                return new Promise(function(res, rej) { resolve = res; });\
            };\
        ").call(Some(&audio_ctx), Some(&audio_callback)).unwrap().into_fn()
        };
        let promise =
            unsafe { reset_promise.call(None, None).unwrap().into_promise() };

        Some(Self {
            promise,
            reset_promise,
            tmp_audio_l,
            tmp_audio_r,
            sample_rate,
            buffers,
            is_buffer_b,
            fn_left,
            fn_right,
            ready: false,
            _phantom,
        })
    }

    pub(crate) fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<f64> {
        if let Poll::Ready(_result) = self.promise.poll() {
            debug_assert_eq!(unsafe { _result.into_i32() }, 1024);
            self.promise = unsafe {
                self.reset_promise.call(None, None).unwrap().into_promise()
            };
            self.ready = true;
            Poll::Ready(self.sample_rate)
        } else {
            Poll::Pending
        }
    }

    pub(crate) fn play_last(&mut self, audio: &[F]) -> usize {
        //
        if self.ready == false {
            return 0;
        }
        self.ready = false;

        //
        let silence = [F::default(); 1024];
        let audio = if audio.len() < 1024 { &silence } else { audio };

        //
        let buffer = JsVar::from_i32(if self.is_buffer_b { 1 } else { 0 });
        self.is_buffer_b = !self.is_buffer_b;
        for (i, sample) in audio.iter().enumerate() {
            if i == 1024 {
                break;
            }
            let (l, r) = sample.into_f64x2();
            self.tmp_audio_l[i] = l;
            self.tmp_audio_r[i] = r;
        }
        unsafe {
            let buffer_l = self
                .fn_left
                .call(Some(&self.buffers), Some(&buffer))
                .unwrap();
            let buffer_r = self
                .fn_right
                .call(Some(&self.buffers), Some(&buffer))
                .unwrap();
            buffer_l.write_doubles(&self.tmp_audio_l);
            buffer_r.write_doubles(&self.tmp_audio_r);
        }

        if audio.as_ptr() == silence.as_ptr() {
            1024 // audio.len()
        } else {
            1024
        }
    }
}

pub(crate) struct Recorder<F: Frame> {
    // Sample rate of the speakers
    sample_rate: f64,
    // JavaScript Promise
    promise: JsPromise<JsVar>,
    // JavaScript Function for resetting promise
    reset_promise: JsFn,
    // The ready value from the promise resolving.
    ready: bool,
    // JavaScript output: audio array
    array: JsVar,
    // Input buffer
    buffer: Vec<f64>,
    _phantom: PhantomData<F>,
}

impl<F: Frame> Recorder<F> {
    pub(crate) fn new() -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let audio_ctx = unsafe {
            JsFn::new(
                "return new (window.AudioContext\
                ||window.webkitAudioContext)();",
            )
            .call(None, None)
            .unwrap()
        };
        let sample_rate: f64 = unsafe {
            JsFn::new("return param_a.sampleRate;")
                .call(Some(&audio_ctx), None)
                .unwrap()
                .into_f64()
                .try_into()
                .unwrap()
        };
        let array = unsafe {
            JsFn::new("return Array(1024);").call(None, None).unwrap()
        };
        let reset_promise = unsafe {
            JsFn::new(
                "\
            var resolve;\
            if (navigator.mediaDevices) {\
                navigator.mediaDevices.getUserMedia ({audio: true})\
                .then(function(stream) {\
                    let options= {mediaStreamTrack:stream.getAudioTracks()[0]};\
                    let source = new MediaStreamTrackAudioSourceNode(param_a,\
                        options);\
                    var scriptNode = param_a.createScriptProcessor(1024, 1, 0);\
                    scriptNode.onaudioprocess = function(ev) {\
                        var inputData = ev.inputBuffer.getChannelData(0);\
                        for (var sample = 0; sample < 1024; sample++) {\
                            param_b[sample] = inputData[sample];\
                        }\
                        resolve(1024);\
                    };\
                    source.connect(scriptNode);\
                });\
            } else {\
                return null;\
            }\
            return function(a, b) {\
                return new Promise(function(res, rej) { resolve = res; });\
            };\
        ",
            )
            .call(Some(&audio_ctx), Some(&array))?
            .into_fn()
        };
        let promise =
            unsafe { reset_promise.call(None, None).unwrap().into_promise() };

        Some(Self {
            array,
            ready: false,
            buffer: Vec::new(),
            _phantom,
            promise,
            sample_rate,
            reset_promise,
        })
    }

    pub(crate) fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<f64> {
        if let Poll::Ready(_result) = self.promise.poll() {
            unsafe {
                self.array.read_doubles(&mut self.buffer);
            }
            self.ready = true;
            self.promise = unsafe {
                self.reset_promise.call(None, None).unwrap().into_promise()
            };
            Poll::Ready(self.sample_rate)
        } else {
            Poll::Pending
        }
    }

    pub(crate) fn record_last(&mut self, audio: &mut Vec<F>) {
        if self.ready {
            for i in self.buffer.iter().cloned() {
                audio.push(F::from_f64x2(i, i));
            }
        }
        self.ready = false;
    }
}
