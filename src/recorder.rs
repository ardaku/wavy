// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ffi;
use crate::frame::Frame;

/// Audio Recorder (Microphone input).  When polled as a future, returns the
/// sample rate of the device.
#[allow(missing_debug_implementations)]
pub struct Recorder<F: Frame>(ffi::Recorder<F>);

impl<F: Frame> Recorder<F> {
    /// Create a new audio recorder.
    pub fn new() -> Option<Self> {
        Some(Recorder(ffi::Recorder::new()?))
    }

    /// Record audio from connected microphones.  Get a future that writes the
    /// newly recorded audio frames into a `Vec`.
    pub fn record_last(&mut self, audio: &mut Vec<F>) {
        // This checks to see if any samples can be added (capacity is used).
        // If not, reserve space.
        if audio.len() + 1024 > audio.capacity() {
            audio.reserve(audio.capacity() + 1024);
        }
        self.0.record_last(audio);
    }
}

impl<F: Frame + Unpin> Future for Recorder<F> {
    type Output = f64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.poll(cx)
    }
}
