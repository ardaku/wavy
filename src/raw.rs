// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! This file contains traits that correspond to implementations for different
//! platforms.  When adding new targets, this will need to be a reference.

#![allow(unsafe_code)]

use std::array::IntoIter;
use std::fmt::{Display, Formatter, Result};
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Once;
use std::task::Context;
use std::task::Poll;
use std::num::NonZeroU32;

#[allow(unused_attributes)]
#[cfg_attr(
    any(target_family="wasm", target_arch = "asmjs"), // FIXME: is asmjs needed?
    cfg_attr(target_os = "wasi", path = "raw/wasi.rs"),
    cfg_attr(target_os = "ardaku", path = "raw/ardaku.rs"),
    cfg_attr(
        any(target_os = "unknown", target_os = "emscripten"),
        path = "raw/dom.rs"
    )
)]
#[cfg_attr(
    not(any(target_arch = "wasm32", target_arch = "asmjs")),
    cfg_attr(target_os = "linux", path = "raw/linux.rs"),
    cfg_attr(target_os = "android", path = "raw/android.rs"),
    cfg_attr(target_os = "macos", path = "raw/macos.rs"),
    cfg_attr(target_os = "ios", path = "raw/ios.rs"),
    cfg_attr(target_os = "windows", path = "raw/windows.rs"),
    cfg_attr(target_os = "fuchsia", path = "raw/fuchsia.rs"),
    cfg_attr(target_os = "redox", path = "raw/redox.rs"),
    cfg_attr(
        any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "bitrig",
            target_os = "openbsd",
            target_os = "netbsd"
        ),
        path = "raw/bsd.rs",
    )
)]
#[path = "raw/unknown.rs"]
mod ffi;

/*pub(crate) fn record<Chan: Channel, const N: usize>(
    microphone: &mut dyn Microphone,
    audio: &mut Audio<Chan, N>
)
    where Chan: From<Ch32>
{
    let source = microphone.record(N);
    assert_eq!(source.len() / N, audio.len());
    for (srcf, dstf) in source.chunks(N).zip(audio.iter_mut()) {
        for (src, dst) in srcf.iter().zip(dstf.channels_mut().iter_mut()) {
            *dst = Chan::from(Ch32::new(*src));
        }
    }
}*/

/*pub(crate) fn play<Chan: Channel, S: Stream<Chan>, const N: usize>(
    speakers: &mut dyn Speakers,
    stream: &mut S,
)
    where Ch32: From<Chan>
{
    let buffer = speakers.play(N);

    unsafe {
        let boxed_samples = Box::<[f32]>::from_raw(buffer);
        let mut audio = Audio::<Ch32, N>::with_f32_buffer(speakers.sample_rate(), boxed_samples);
        stream.sink(&mut audio);
        std::mem::forget(audio);
    }
}*/

#[derive(Clone)]
struct FakeSpeakers;

impl Display for FakeSpeakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "FakeSpeakers")
    }
}

impl Speakers for FakeSpeakers {}

#[derive(Clone)]
struct FakeMicrophone;

impl Display for FakeMicrophone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "FakeMicrophone")
    }
}

impl Microphone for FakeMicrophone {}

/// Speakers implementation.
pub(crate) trait Speakers: Send + Display {
    /// Get the name of these speakers.
    fn name(&self) -> &str {
        "Unknown"
    }

    /// Poll for reference to internal audio buffer to write to.
    fn poll(&self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    /// Get the actual sample rate of the microphone.
    fn sample_rate(&self) -> u32 {
        crate::consts::SAMPLE_RATE.into()
    }

    /// Play audio though the speakers.
    ///
    /// Wavy writes to the buffer, and it's expected to be played after written.
    fn play<'a>(&'a self) -> [&'a mut [f32]; 8] {
        [&mut [], &mut [], &mut [], &mut [], &mut [], &mut [], &mut [], &mut []]
    }
}

/// Microphone implementation.
pub(crate) trait Microphone: Send + Display {
    /// Get the name of this microphone.
    fn name(&self) -> &str {
        "Unknown"
    }

    /// Poll for reference to internal audio buffer to use as a `Stream`.
    fn poll(&self, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    /// Get the actual sample rate of the microphone.
    fn sample_rate(&self) -> u32 {
        crate::consts::SAMPLE_RATE.into()
    }

    /// Record audio from the microphone.
    ///
    /// Wavy writes to the buffer, and it's expected to be read after written.
    fn record(&self, _sample_rate: NonZeroU32, mut audio: [&mut [f32]; 8]) {
        for channel in audio.iter_mut() {
            for sample in channel.iter_mut() {
                *sample = 0.0;
            }
        }
    }
}

pub(crate) struct NativeIterator<Item>(Box<dyn Iterator<Item = Item>>);

impl<Item> Iterator for NativeIterator<Item> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) trait Global {
    /// Create a new iterator of `Speakers`
    fn query_speakers(&'static self) -> NativeIterator<Rc<dyn Speakers>> {
        NativeIterator(Box::new(IntoIter::new([Rc::new(FakeSpeakers) as _])))
    }

    /// Create a new iterator of `Microphone`s
    fn query_microphones(&'static self) -> NativeIterator<Rc<dyn Microphone>> {
        NativeIterator(Box::new(IntoIter::new([Rc::new(FakeMicrophone) as _])))
    }
}

struct FakeGlobal;

impl Global for FakeGlobal {}

static START: Once = Once::new();
static mut GLOBAL: MaybeUninit<Box<dyn Global>> = MaybeUninit::uninit();

pub(crate) fn global() -> &'static dyn Global {
    START.call_once(|| unsafe {
        let global = ffi::global().unwrap_or(Box::new(FakeGlobal));
        std::ptr::write(GLOBAL.as_mut_ptr(), global);
    });

    unsafe { &*(*GLOBAL.as_ptr()) }
}
