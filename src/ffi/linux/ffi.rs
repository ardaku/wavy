// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

mod asound;
mod microphone;
mod speakers;

// Implementation Expectations:
use asound::{
    device_list::{open, pcm_hw_params, AudioDevice, SoundDevice, DEFAULT},
    PollFd, SndPcmAccess, SndPcmFormat, SndPcmMode, SndPcmState, SndPcmStream,
};

pub(crate) use asound::device_list::device_list;
pub(crate) use microphone::{Microphone, MicrophoneStream};
pub(crate) use speakers::{Speakers, SpeakersSink};
