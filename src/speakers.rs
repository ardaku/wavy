// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use fon::{
    chan::{Ch16, Ch32, Ch64},
    sample::Sample,
    Audio, Sink,
};

use crate::ffi::Speakers as SpeakersSys;

/// Play audio samples through speaker system.
///
/// # 440 HZ Sine Wave Example
/// **note:** This example depends on `twang = "0.3"` to synthesize the sine
/// wave.
/// ```no_run
/// use fon::mono::Mono64;
/// use pasts::prelude::*;
/// use std::cell::RefCell;
/// use twang::Synth;
/// use wavy::Speakers;
///
/// /// The program's shared state.
/// struct State {}
///
/// /// Speakers task (play sine wave).
/// async fn speakers(state: &RefCell<State>) {
///     // Connect to system's speaker(s)
///     let mut speakers = Speakers::<Mono64>::new();
///     // Create a new synthesizer
///     let mut synth = Synth::new();
///
///     loop {
///         // 1. Wait for speaker to need more samples.
///         let audio = speakers.play().await;
///         // 2. Borrow shared state mutably
///         let _state = state.borrow_mut();
///         // 3. Generate and write samples into speaker buffer.
///         synth.gen(audio, |fc| fc.freq(440.0).sine().gain(0.7));
///     }
/// }
///
/// /// Program start.
/// async fn start() {
///     // Initialize shared state.
///     let state = RefCell::new(State {});
///     // Create speaker task.
///     let mut speakers = speakers(&state);
///     // Wait for first task to complete.
///     [speakers.fut()].select().await;
/// }
///
/// /// Start the async executor.
/// fn main() {
///     pasts::spawn(start);
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Speakers<S: Sample + Unpin>
where
    Ch16: From<S::Chan>,
    Ch32: From<S::Chan>,
    Ch64: From<S::Chan>,
{
    speakers: SpeakersSys<S>,
    audiobuf: Audio<S>,
}

impl<S: Sample + Unpin> Speakers<S>
where
    Ch16: From<S::Chan>,
    Ch32: From<S::Chan>,
    Ch64: From<S::Chan>,
{
    /// Connect to the speaker system.
    ///
    /// # Panics
    /// - If already connected to the speaker system.
    #[allow(clippy::new_without_default)] // Because it may panic
    pub fn new() -> Self {
        let (speakers, sample_rate) = SpeakersSys::connect();
        let audiobuf = Audio::with_silence(sample_rate, 1024);
        Self { speakers, audiobuf }
    }

    /// Get the speakers' sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.audiobuf.sample_rate()
    }

    /// Play audio through speakers.  Returns mutable reference to next audio
    /// buffer to play.  If you don't overwrite the buffer, it will keep playing
    /// whatever was last written into it.
    pub async fn play(&mut self) -> impl Sink<S> + '_ {
        self.speakers.play(&self.audiobuf);
        (&mut self.speakers).await;
        self.audiobuf.sink(..)
    }
}
