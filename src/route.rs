// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use crate::{
    ffi::{self, device_list, AudioDst, AudioSrc},
    Microphone, Speakers,
};
use std::fmt::{Display, Error, Formatter};

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
    pub fn connect(&self) -> Option<Microphone> {
        Some(Microphone (
             ffi::Microphone::new(&self)?,
        ))
    }
}

impl Display for MicrophoneId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

/// ID of an available speaker, pair of speakers, or surround sound system, or
/// other audio output.
#[derive(Debug, Default)]
pub struct SpeakersId(pub(crate) AudioDst);

impl SpeakersId {
    /// Query available audio destinations.
    pub fn query() -> Vec<Self> {
        device_list(Self)
    }

    /// Connect to this speaker.  Returns `None` if the speaker is unplugged.
    pub fn connect(&self) -> Option<Speakers> {
        Some(Speakers(ffi::Speakers::connect(&self)?))
    }
}

impl Display for SpeakersId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}
