// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::ffi;
use fon::{
    chan::{Ch16, Channel},
    sample::Sample1,
    Stream,
};

/// Record audio samples from a microphone.
#[allow(missing_debug_implementations)]
pub struct Microphone<C: Channel + Unpin> {
    microphone: ffi::Microphone<C>,
}

impl<C: Channel + Unpin + From<Ch16>> Microphone<C> {
    /// Connect to a microphone.  Unlike `Speakers`, you may call this multiple
    /// times to connect to multiple devices.
    pub fn new() -> Option<Self> {
        Some(Self {
            microphone: ffi::Microphone::new()?,
        })
    }

    /// Get the microphone's sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.microphone.sample_rate()
    }

    /// Record audio from connected microphone.  Returns new audio frames as an
    /// `Audio` buffer in the requested format.
    pub async fn record(&mut self) -> impl Stream<Sample1<C>> + '_ {
        (&mut self.microphone).await;
        self.microphone.record()
    }
}
