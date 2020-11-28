// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::fmt::{Formatter, Error, Display};
use crate::ffi::{AudioSrc as AudioSrcSys, AudioDst as AudioDstSys, device_list};

/// An available audio input.
#[derive(Debug)]
pub struct MicrophoneId(AudioSrcSys);

impl MicrophoneId {
    /// Query available audio sources.
    pub fn query() -> Vec<Self> {
        device_list(|a: AudioSrcSys| Self(a))
    }
}

impl Display for MicrophoneId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

/// An available audio output.
#[derive(Debug)]
pub struct SpeakerId(AudioDstSys);

impl SpeakerId {
    /// Query available audio destinations.
    pub fn query() -> Vec<Self> {
        device_list(|a: AudioDstSys| Self(a))
    }
}

impl Display for SpeakerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}
