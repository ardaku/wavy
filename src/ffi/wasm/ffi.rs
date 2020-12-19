// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::task::Waker;

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    AudioContext, AudioContextOptions, AudioDestinationNode,
    AudioProcessingEvent, MediaStreamAudioSourceNode, ScriptProcessorNode,
};

/// 1024 samples per period.
const PERIOD: u16 = 1024;
/// 48KHz
const SAMPLE_RATE: u32 = 48_000;

/// Global State of AudioContext.
///
/// There are 4 possible states:
///  - No devices
///  - A speaker
///  - One or more microphones
///  - Both a speaker and one or more microphones
struct State {
    /// The JavaScript audio context, lazily initialized.
    context: Option<AudioContext>,
    /// Speaker, if any.
    speaker: Option<AudioDestinationNode>,
    /// Microphones, if any.
    microphone: Vec<MediaStreamAudioSourceNode>,
    /// Input channel buffer.
    i_buffer: [f32; PERIOD as usize],
    /// Left output channel buffer.
    l_buffer: [f32; PERIOD as usize],
    /// Right output channel buffer.
    r_buffer: [f32; PERIOD as usize],
    /// The processor node that wakes and executes futures.  Though this API is
    /// deprecated, the new API does not work on Safari (yet).  This currently
    /// works on all browsers.  Once browser support changes, this should be
    /// changed to use `AudioWorkletNode`.
    proc: Option<ScriptProcessorNode>,
    /// Waker from speaker future
    speaker_waker: Option<Waker>,
    /// Waker from microphone future.
    mics_waker: Option<Waker>,
    ///
    played: bool,
    ///
    recorded: bool,
}

impl State {
    fn lazy_init(&mut self) {
        // AudioContext
        if state().context.is_none() {
            state().context = Some(
                AudioContext::new_with_context_options(
                    &AudioContextOptions::new(), //.sample_rate(SAMPLE_RATE as f32),
                )
                .expect("Couldn't initialize AudioContext"),
            );
        }

        // ScriptProcessorNode
        if self.proc.is_none() {
            let proc = self
                .context
                .as_ref()
                .unwrap()
                .create_script_processor_with_buffer_size(PERIOD.into())
                .unwrap();
            #[allow(trivial_casts)] // Actually needed here.
            let js_function: Closure<dyn Fn(AudioProcessingEvent)> =
                Closure::wrap(Box::new(move |event| {
                    // If a microphone is being `.await`ed, wake the thread with
                    // the input buffer.
                    if let Some(waker) = state().mics_waker.take() {
                        // Grab the AudioBuffer.
                        let inbuf = event
                            .input_buffer()
                            .expect("Failed to get input buffer");
                        // Read microphone input.
                        inbuf
                            .copy_from_channel(&mut state().i_buffer, 0)
                            .unwrap();
                        // Set future to complete.
                        state().recorded = true;
                        // Wake the microphone future.
                        waker.wake();
                    }

                    // If the speakers are being `.await`ed, wake the thread to
                    // fill the output buffer.
                    if let Some(waker) = state().speaker_waker.take() {
                        // Set future to complete.
                        state().played = true;
                        // Wake the speaker future to generate audio data.
                        waker.wake();
                        // Grab the AudioBuffer.
                        let out = event
                            .output_buffer()
                            .expect("Failed to get output buffer");
                        // Write speaker output.
                        out.copy_to_channel(&mut state().l_buffer, 0).unwrap();
                        out.copy_to_channel(&mut state().r_buffer, 1).unwrap();
                    }
                }));
            proc.set_onaudioprocess(Some(js_function.as_ref().unchecked_ref()));
            js_function.forget();
            self.proc = Some(proc);
        }
    }
}

/// Global state of AudioContext.
static mut STATE: State = State {
    context: None,
    speaker: None,
    microphone: Vec::new(),
    i_buffer: [0.0; PERIOD as usize],
    l_buffer: [0.0; PERIOD as usize],
    r_buffer: [0.0; PERIOD as usize],
    proc: None,
    speaker_waker: None,
    mics_waker: None,
    played: false,
    recorded: false,
};

/// Since Web Assembly can only have one thread, accessing our global state is
/// safe.
#[allow(unsafe_code)]
#[inline(always)]
fn state() -> &'static mut State {
    unsafe { &mut STATE }
}

mod microphone;
mod speakers;
mod device_list;

pub(super) use microphone::Microphone;
pub(super) use speakers::Speakers;
pub(crate) use device_list::{device_list, AudioDst, AudioSrc};

/* use cala_core::os::web::{JsFn, JsPromise, JsVar};
use fon::{
    chan::{Ch64, Channel},
    sample::{Sample, Sample1},
    stereo::Stereo64,
    Audio, Resampler, Stream,
};
use std::{
    convert::TryInto,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) struct Speakers<S: Sample> {
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
    promise: JsPromise<JsVar>,
    // JavaScript Function for resetting promise
    reset_promise: JsFn,
    //
    ready: bool,
    _phantom: PhantomData<S>,
}

impl<S: Sample> Speakers<S> {
    pub(crate) fn connect() -> (Self, u32) {
        let _phantom = PhantomData::<S>;
        let audio_ctx = unsafe {
            JsFn::new(
                "return new (window.AudioContext\
                ||window.webkitAudioContext)();",
            )
            .call(None, None)
            .unwrap()
        };
        let sample_rate: u32 = unsafe {
            JsFn::new("return param_a.sampleRate;")
                .call(Some(&audio_ctx), None)
                .unwrap()
                .into_i32()
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

        (
            Self {
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
            },
            sample_rate,
        )
    }

    pub(crate) fn play(&mut self, audio: &Audio<S>) -> usize
    where
        Ch64: From<S::Chan>,
    {
        //
        if self.ready == false {
            return 0;
        }
        self.ready = false;

        //
        let silence = Audio::with_silence(self.sample_rate, 1024);
        let audio = if audio.len() < 1024 { &silence } else { audio };

        //
        let buffer = JsVar::from_i32(if self.is_buffer_b { 1 } else { 0 });
        self.is_buffer_b = !self.is_buffer_b;
        for (i, sample) in audio.iter().enumerate() {
            if i == 1024 {
                break;
            }
            let sample: Stereo64 = sample.convert();
            self.tmp_audio_l[i] = sample.channels()[0].to_f64();
            self.tmp_audio_r[i] = sample.channels()[1].to_f64();
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

        1024
    }
}

impl<S: Sample> Future for &mut Speakers<S> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Poll::Ready(_result) = this.promise.poll() {
            debug_assert_eq!(unsafe { _result.into_i32() }, 1024);
            this.promise = unsafe {
                this.reset_promise.call(None, None).unwrap().into_promise()
            };
            this.ready = true;
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub(crate) struct Microphone<C: Channel> {
    // JavaScript Promise
    promise: JsPromise<JsVar>,
    // JavaScript Function for resetting promise
    reset_promise: JsFn,
    // The ready value from the promise resolving.
    ready: bool,
    // JavaScript output: audio array
    array: JsVar,
    //
    stream: MicrophoneStream<C>,
    _phantom: PhantomData<C>,
}

impl<C: Channel> Microphone<C> {
    pub(crate) fn new() -> Option<Self> {
        let _phantom = PhantomData::<C>;
        let audio_ctx = unsafe {
            JsFn::new(
                "return new (window.AudioContext\
                ||window.webkitAudioContext)();",
            )
            .call(None, None)
            .unwrap()
        };
        let sample_rate: u32 = unsafe {
            JsFn::new("return param_a.sampleRate;")
                .call(Some(&audio_ctx), None)
                .unwrap()
                .into_i32()
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
            stream: MicrophoneStream {
                buffer: Vec::new(),
                index: 0,
                resampler: Resampler::new(),
                sample_rate,
            },
            _phantom,
            promise,
            reset_promise,
        })
    }

    pub(crate) fn record(&mut self) -> &mut MicrophoneStream<C> {
        self.ready = false;
        &mut self.stream
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.stream.sample_rate
    }
}

impl<C: Channel> Future for Microphone<C> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Poll::Ready(_result) = this.promise.poll() {
            // Reset
            this.stream.buffer.clear();
            this.stream.index = 0;
            //
            unsafe {
                this.array.read_doubles(&mut this.stream.buffer);
            }
            this.ready = true;
            this.promise = unsafe {
                this.reset_promise.call(None, None).unwrap().into_promise()
            };
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub(crate) struct MicrophoneStream<C: Channel> {
    // Sample rate of the speakers
    sample_rate: u32,
    // Input buffer
    buffer: Vec<f64>,
    //
    index: usize,
    // Stream's resampler
    resampler: Resampler<Sample1<C>>,
}

impl<C> Stream<Sample1<C>> for &mut MicrophoneStream<C>
where
    C: Channel,
{
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn stream_sample(&mut self) -> Option<Sample1<C>> {
        if self.index == self.buffer.len() {
            return None;
        }
        let sample: C = C::from(self.buffer[self.index]);
        self.index += 1;
        Some(Sample1::new(sample))
    }

    fn resampler(&mut self) -> &mut Resampler<Sample1<C>> {
        &mut self.resampler
    }
}*/
