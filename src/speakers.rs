// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use fon::{chan::Ch16, sample::Sample, Audio};

use crate::ffi::Speakers as SpeakersSys;

/// Audio Player (Speaker output).  When polled as a future, returns the sample
/// rate of the device.
#[allow(missing_debug_implementations)]
pub struct Speakers<S: Sample + Unpin>
where
    Ch16: From<S::Chan>,
{
    speakers: SpeakersSys<S>,
    audiobuf: Audio<S>,
}

impl<S: Sample + Unpin> Speakers<S>
where
    Ch16: From<S::Chan>,
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

    /// Play audio through speakers.  Returns mutable reference to next audio
    /// buffer to play.  If you don't overwrite the buffer, it will keep playing
    /// whatever was last written into it.
    pub async fn play(&mut self) -> &mut Audio<S> {
        self.speakers.play(&self.audiobuf);
        (&mut self.speakers).await;
        &mut self.audiobuf
    }
}
