// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use crate::ffi;
use fon::{
    chan::Channel,
    mono::Mono,
    Stream,
};

/// Record audio samples from a microphone.
#[allow(missing_debug_implementations)]
pub struct Microphone<C: Channel> {
    pub(super) microphone: ffi::Microphone<C>,
}

impl<C: Channel> Microphone<C> {
    /// Get the microphone's sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.microphone.sample_rate()
    }

    /// Record audio from connected microphone.  Returns new audio frames as an
    /// `Audio` buffer in the requested format.
    pub async fn record(&mut self) -> impl Stream<Mono<C>> + '_ {
        (&mut self.microphone).await;
        self.microphone.record()
    }
}
