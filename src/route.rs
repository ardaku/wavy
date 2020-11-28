// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{ffi::{self, device_list, AudioDst, AudioSrc}, Speaker, Microphone};
use std::fmt::{Display, Error, Formatter};
use fon::{Audio, chan::{Ch16, Ch32, Ch64, Channel}, sample::Sample};

/// ID of an available microphone, or other audio input.
#[derive(Debug, Default)]
pub struct MicrophoneId(pub(crate) AudioSrc);

impl MicrophoneId {
    /// Query available audio sources.
    pub fn query() -> Vec<Self> {
        device_list(Self)
    }
    
    /// Connect to this microphone.  Returns `None` if the microphone is
    /// unplugged.
    pub fn connect<C: Channel + Unpin>(self) -> Option<Microphone<C>> {
        Some(Microphone {
            microphone: ffi::Microphone::new(self)?,
        })
    }
}

impl Display for MicrophoneId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

/// ID of an available speaker, or other audio output.
#[derive(Debug, Default)]
pub struct SpeakerId(pub(crate) AudioDst);

impl SpeakerId {
    /// Query available audio destinations.
    pub fn query() -> Vec<Self> {
        device_list(Self)
    }
    
    /// Connect to this speaker.  Returns `None` if the speaker is unplugged.
    pub fn connect<S: Sample + Unpin>(self) -> Option<Speaker<S>>
    where
        Ch16: From<S::Chan>,
        Ch32: From<S::Chan>,
        Ch64: From<S::Chan>,
    {
        let (speakers, sample_rate) = ffi::Speakers::connect(self)?;
        let audiobuf = Audio::with_silence(sample_rate, 1024);
        Some(Speaker { speakers, audiobuf })
    }
}

impl Display for SpeakerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}
