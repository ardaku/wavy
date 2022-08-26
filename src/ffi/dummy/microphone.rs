// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::{
    fmt::{Display, Error, Formatter},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{chan::Ch32, Frame, Stream};

use super::SoundDevice;

pub(crate) struct Microphone();

impl SoundDevice for Microphone {
    const INPUT: bool = true;
}

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Default")
    }
}

impl Default for Microphone {
    fn default() -> Self {
        Microphone()
    }
}

impl Microphone {
    pub(crate) fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<'_, F> {
        MicrophoneStream(PhantomData)
    }

    pub(crate) fn channels(&self) -> u8 {
        1
    }
}

impl Future for Microphone {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub(crate) struct MicrophoneStream<'a, F: Frame<Chan = Ch32>>(
    PhantomData<&'a F>,
);

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<'_, F> {
    fn sample_rate(&self) -> Option<f64> {
        Some(crate::consts::SAMPLE_RATE.into())
    }

    fn len(&self) -> Option<usize> {
        Some(crate::consts::PERIOD.into())
    }
}
