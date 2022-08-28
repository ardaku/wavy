// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

mod asound;
mod microphone;
mod speakers;

// Implementation Expectations:
pub(crate) use asound::device_list::device_list;
use asound::{
    device_list::{open, pcm_hw_params, AudioDevice, SoundDevice, DEFAULT},
    PollFd, SndPcmAccess, SndPcmFormat, SndPcmMode, SndPcmState, SndPcmStream,
};
pub(crate) use microphone::{Microphone, MicrophoneStream};
pub(crate) use speakers::{Speakers, SpeakersSink};
