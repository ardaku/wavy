// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::ffi::{device_list, AudioDst, AudioSrc};
use std::fmt::{Display, Error, Formatter};

/// ID of an available microphone.
#[derive(Debug, Default)]
pub struct MicrophoneId(pub(crate) AudioSrc);

impl MicrophoneId {
    /// Query available audio sources.
    pub fn query() -> Vec<Self> {
        device_list(Self)
    }
}

impl Display for MicrophoneId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

/// ID of an available speaker.
#[derive(Debug, Default)]
pub struct SpeakerId(pub(crate) AudioDst);

impl SpeakerId {
    /// Query available audio destinations.
    pub fn query() -> Vec<Self> {
        device_list(Self)
    }
}

impl Display for SpeakerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}
