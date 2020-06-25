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

/// Audio Player (Speaker output).  When polled as a future, returns the sample
/// rate of the device.
#[allow(missing_debug_implementations)]
pub struct Player<F: Frame>(ffi::Player<F>);

impl<F: Frame> Player<F> {
    /// Connect to the speaker system.
    pub fn new() -> Option<Player<F>> {
        Some(Player(ffi::Player::new()?))
    }

    /// Play from a slice of audio samples.  Returns a future that returns the
    /// number of audio samples actually played.
    pub fn play_last(&mut self, audio: &[F]) -> usize {
        self.0.play_last(audio)
    }
}

impl<F: Frame + Unpin> Future for Player<F> {
    type Output = f64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.poll(cx)
    }
}
