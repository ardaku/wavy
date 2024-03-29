// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::task::Waker;

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    AudioContext, AudioDestinationNode, AudioProcessingEvent,
    MediaStreamAudioSourceNode, ScriptProcessorNode,
};

use crate::consts::BUFFER_SIZE;

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
    i_buffer: [f32; BUFFER_SIZE as usize],
    /// Left output channel buffer.
    l_buffer: [f32; BUFFER_SIZE as usize],
    /// Right output channel buffer.
    r_buffer: [f32; BUFFER_SIZE as usize],
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
    /// Sample rate cached across FFI boundary.
    sample_rate: Option<f64>,
}

impl State {
    fn lazy_init(&mut self) {
        // AudioContext
        if state().context.is_none() {
            let audio_context =
                AudioContext::new().expect("Couldn't initialize AudioContext");

            state().sample_rate = Some(audio_context.sample_rate().into());

            state().context = Some(audio_context);
        }

        // ScriptProcessorNode
        if self.proc.is_none() {
            let proc = self
                .context
                .as_ref()
                .unwrap()
                .create_script_processor_with_buffer_size(BUFFER_SIZE.into())
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
    i_buffer: [0.0; BUFFER_SIZE as usize],
    l_buffer: [0.0; BUFFER_SIZE as usize],
    r_buffer: [0.0; BUFFER_SIZE as usize],
    proc: None,
    speaker_waker: None,
    mics_waker: None,
    played: false,
    recorded: false,
    sample_rate: None,
};

/// Since Web Assembly can only have one thread, accessing our global state is
/// safe.
#[allow(unsafe_code)]
#[inline(always)]
fn state() -> &'static mut State {
    unsafe { &mut STATE }
}

mod device_list;
mod microphone;
mod speakers;

pub(crate) use device_list::device_list;
use device_list::SoundDevice;
pub(super) use microphone::{Microphone, MicrophoneStream};
pub(super) use speakers::{Speakers, SpeakersSink};
