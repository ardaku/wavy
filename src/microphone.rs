// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::env::Recording;
use flume::Receiver;
use fon::chan::Channel;
use fon::{Frame, Sink};
use std::fmt::{Debug, Display, Formatter, Result};
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Record audio samples from a microphone.
///
/// Microphones are always implemented with support for 7.1 surround sound.  The
/// channel order is compatible with 5.1 surround and 2.0 stereo.  This is
/// usually unnecessary, but can be useful for virtual microphones (that
/// capture audio from speakers, or some other source).  For most microphones,
/// all audio will be placed in the [`FrontL`](fon::pos::FrontL) channel.
pub struct Microphone(Receiver<Recording>);

impl Default for Microphone {
    fn default() -> Self {
        crate::env::query_microphones().recv().unwrap()
    }
}

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

impl Debug for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<'a> Future for Microphone {
    type Output = Recorder;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0.recv_async())
            .poll(cx)
            .map(|x| Recorder(x.unwrap()))
    }
}

/// A stream of recorded audio samples from a microphone.
pub struct Recorder(Recording);

impl Debug for Recorder {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        write!(fmt, "Recorder(rate: {:?})", self.sample_rate())
    }
}

impl Recorder {
    /// Get the sample rate of the stream.
    pub fn sample_rate(&self) -> NonZeroU32 {
        self.0.sample_rate()
    }

    /// Pipe audio from microphone through stream resampler into sink.
    pub fn pipe<S, Chan, const CH: usize>(&self, mut sink: S)
    where
        S: Sink<Chan, CH>,
        Chan: Channel,
    {
        // Make sure sink is at the correct sample rate.
        assert_eq!(self.sample_rate(), sink.sample_rate());

        // Get environment-dependent frame iterator.
        self.0.record(&mut |frame_iter| {
            sink.sink_with(frame_iter.map(|in_frame| {
                let mut out_frame = Frame::<Chan, CH>::default();
                for (in_, out) in in_frame
                    .channels()
                    .iter()
                    .zip(out_frame.channels_mut().iter_mut())
                {
                    *out = Chan::from(in_.to_f32());
                }
                out_frame
            }));
        });
    }
}
