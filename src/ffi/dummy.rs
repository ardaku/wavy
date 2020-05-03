// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::frame::Frame;
use std::marker::PhantomData;
use std::task::Context;
use std::task::Poll;

pub(crate) struct Player<F: Frame> {
    _phantom: PhantomData<F>,
}

impl<F: Frame> Player<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let _ = sr;

        None
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let _ = cx;
    
        Poll::Pending
    }

    pub(crate) fn play_last(&mut self, audio: &[F]) -> usize {
        let _ = audio;
    
        0 // 0 frames were written.
    }
}

pub(crate) struct Recorder<F: Frame> {
    _phantom: PhantomData<F>,
}

impl<F: Frame> Recorder<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let _ = sr;

        None
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let _ = cx;
    
        Poll::Pending
    }

    pub(crate) fn record_last(&mut self, audio: &mut Vec<F>) {
        let _ = audio;
    }
}
