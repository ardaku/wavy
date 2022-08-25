// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! Linux platform support

use super::Support;
use asound::{SndPcmStream, AudioDevice};
use crate::Microphone;
use crate::Speakers;
use lookit::Lookit;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::collections::HashSet;

mod asound;
mod devices;

pub(crate) struct Platform;

pub(crate) struct PlatformSpeakersQuery(Lookit, Vec<AudioDevice>, HashSet<String>);
pub(crate) struct PlatformMicrophoneQuery(Lookit, Vec<AudioDevice>, HashSet<String>);

pub(crate) struct PlatformSpeakers(AudioDevice);
pub(crate) struct PlatformMicrophone(AudioDevice);

impl Support for &Platform {
    fn query_speakers(self) -> PlatformSpeakersQuery {
        PlatformSpeakersQuery(Lookit::with_audio(), Vec::new(), HashSet::new())
    }

    fn query_microphones(self) -> PlatformMicrophoneQuery {
        PlatformMicrophoneQuery(Lookit::with_audio(), Vec::new(), HashSet::new())
    }

    fn speakers_name(self, speakers: &PlatformSpeakers) -> &str {
        speakers.0.name.as_str()
    }

    fn microphone_name(self, microphone: &PlatformMicrophone) -> &str {
        microphone.0.name.as_str()
    }
}

impl Future for PlatformSpeakersQuery {
    type Output = Speakers;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = self.get_mut();

        if let Some(audio_device) = this.1.pop() {
            Poll::Ready(Speakers(PlatformSpeakers(audio_device)))
        } else if Pin::new(&mut this.0).poll(cx).is_ready() {
            while Pin::new(&mut this.0).poll(cx).is_ready() {}
            // Get new devices
            this.1 = asound::devices(SndPcmStream::Playback, &mut this.2);
            Pin::new(this).poll(cx)
        } else {
            Poll::Pending
        }
    }
}

impl Future for PlatformMicrophoneQuery {
    type Output = Microphone;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = self.get_mut();

        if let Some(audio_device) = this.1.pop() {
            Poll::Ready(Microphone(PlatformMicrophone(audio_device)))
        } else if Pin::new(&mut this.0).poll(cx).is_ready() {
            while Pin::new(&mut this.0).poll(cx).is_ready() {}
            // Get new devices
            this.1 = asound::devices(SndPcmStream::Capture, &mut this.2);
            Pin::new(this).poll(cx)
        } else {
            Poll::Pending
        }
    }
}
