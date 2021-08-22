// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{Microphone, Speakers};
use flume::Receiver;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Listener for audio devices.  Type parameter `T` can be either `Speaker` or
/// `Microphone`.
pub struct Listener<T: Listenable>(Receiver<T>);

impl<T: Listenable> Listener<T> {
    /// Create a new listener for when new audio devices are plugged in.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Listenable> Default for Listener<T> {
    fn default() -> Self {
        Self(T::new())
    }
}

impl<T: Listenable> Future for Listener<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0.recv_async())
            .poll(cx)
            .map(|x| x.unwrap())
    }
}

pub trait Listenable: Sized {
    fn new() -> Receiver<Self>;
}

impl Listenable for Microphone {
    fn new() -> Receiver<Self> {
        crate::env::query_microphones()
    }
}

impl Listenable for Speakers {
    fn new() -> Receiver<Self> {
        crate::env::query_speakers()
    }
}
