// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(unsafe_code)]

use crate::consts::CHUNK_SIZE;
use crate::raw::{global, Microphone as RawMicrophone};
use fon::chan::Channel;
use fon::{Sink, Stream};
use std::fmt::{Debug, Display, Formatter, Result};
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll};

//
#[repr(transparent)] // to safely transmute from `Rc<dyn RawMicrophone>`.
#[allow(missing_debug_implementations)]
pub struct MicrophoneFuture<const N: usize>(Rc<dyn RawMicrophone>);

impl<const N: usize> Future for MicrophoneFuture<N> {
    type Output = Recorder<N>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(()) = self.0.poll(cx) {
            Poll::Ready(Recorder(Rc::downgrade(&self.0)))
        } else {
            Poll::Pending
        }
    }
}

/// Record audio samples from a microphone.
pub struct Microphone(Rc<dyn RawMicrophone>);

impl Microphone {
    /// Query available audio sources.
    pub fn query() -> impl Iterator<Item = Self> {
        global().query_microphones().map(|x| Self(x))
    }

    /// Record N channels from the microphone.
    ///
    /// Returns a [`Future`](core::future::Future) that outputs a
    /// [`Recorder`](crate::Recorder).
    pub fn record<'a, const N: usize>(
        &'a mut self,
    ) -> &'a mut MicrophoneFuture<N> {
        // Unsafe is needed here to attach the const generic value to the
        // internal microphone type.  Since it's repr(transparent) on the same
        // runtime type and the lifetimes match, this is sound.
        unsafe { std::mem::transmute(&mut self.0) }
    }
}

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Microphone({})", self)
    }
}

impl Default for Microphone {
    fn default() -> Self {
        Self::query().next().unwrap()
    }
}

/// Audio recorder.
pub struct Recorder<const N: usize>(Weak<dyn RawMicrophone>);

impl<const N: usize> Debug for Recorder<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Recorder({})", self.0.upgrade().unwrap())
    }
}

impl<const N: usize> Recorder<N> {
    /// Record audio to a sink.
    pub fn record<Chan, S>(&self, stream: &mut Stream<N>, sink: S)
    where
        Chan: Channel,
        S: Sink<Chan, N>,
    {
        let sample_rate = sink.sample_rate();
        stream.pipe_raw(
            CHUNK_SIZE.into(),
            |audio| self.0.upgrade().unwrap().record(sample_rate, audio),
            sink,
        );
    }
}
