// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::{
    any::TypeId,
    fmt::{Display, Error, Formatter},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    task::{Context, Poll},
};

use fon::{
    chan::{Ch32, Channel},
    mono::Mono32,
    stereo::Stereo32,
    surround::Surround32,
    Frame, Resampler, Sink,
};

use super::SoundDevice;

pub(crate) struct Speakers {
    /// Interleaved buffer (must be de-interleaved for the web).
    buffer: Vec<f32>,
    /// State of resampler.
    resampler: ([Ch32; 6], f64),
    ///
    locked: AtomicBool,
}

impl SoundDevice for Speakers {
    const INPUT: bool = false;
}

impl Display for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Default")
    }
}

impl Default for Speakers {
    fn default() -> Self {
        let state = super::state();

        // Lazily Initialize audio context & processor node.
        state.lazy_init();

        // Check if already connected
        if state.speaker.is_some() {
            panic!("Already connected to speakers!");
        }

        // Initialize speakers.
        state.speaker = Some(state.context.as_mut().unwrap().destination());

        // Connect speakers. FIXME
        state
            .proc
            .as_ref()
            .unwrap()
            .connect_with_audio_node(state.speaker.as_ref().unwrap())
            .ok()
            .unwrap();

        Self {
            buffer: vec![0.0; super::BUFFER_SIZE.into()],
            resampler: ([Ch32::MID; 6], 0.0),
            locked: AtomicBool::new(false),
        }
    }
}

impl Speakers {
    pub(crate) fn play<F: Frame<Chan = Ch32>>(&mut self) -> SpeakersSink<F> {
        // Adjust buffer size depending on type.
        if TypeId::of::<F>() == TypeId::of::<Mono32>() {
            self.buffer.resize(super::BUFFER_SIZE.into(), 0.0);
        } else if TypeId::of::<F>() == TypeId::of::<Stereo32>() {
            self.buffer.resize(super::BUFFER_SIZE as usize * 2, 0.0);
        } else {
            panic!("Attempted to use Speakers with invalid frame type");
        }
        // Convert the resampler to the target speaker configuration.
        let resampler = Resampler::<F>::new(
            Surround32::from_channels(&self.resampler.0[..]).convert(),
            self.resampler.1,
        );
        //
        SpeakersSink(self, resampler, PhantomData)
    }

    pub(crate) fn channels(&self) -> u8 {
        0b0000_0011
    }
}

impl Future for Speakers {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety
        if self.locked.load(SeqCst) {
            eprintln!("Tried to poll speakers before dropping sink");
            std::process::exit(1);
        }

        let state = super::state();
        if state.played {
            state.played = false;
            self.locked.store(true, SeqCst);
            Poll::Ready(())
        } else {
            state.speaker_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub(crate) struct SpeakersSink<F: Frame<Chan = Ch32>>(
    *mut Speakers,
    Resampler<F>,
    PhantomData<F>,
);

impl<F: Frame<Chan = Ch32>> Sink<F> for SpeakersSink<F> {
    fn sample_rate(&self) -> f64 {
        super::state().sample_rate.unwrap()
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.1
    }

    #[allow(unsafe_code)]
    fn buffer(&mut self) -> &mut [F] {
        let speakers = unsafe { self.0.as_mut().unwrap() };

        let data = speakers.buffer.as_mut_ptr().cast();
        let count = super::BUFFER_SIZE.into();
        unsafe { &mut std::slice::from_raw_parts_mut(data, count)[..] }
    }
}

#[allow(unsafe_code)]
impl<F: Frame<Chan = Ch32>> Drop for SpeakersSink<F> {
    fn drop(&mut self) {
        let speakers = unsafe { self.0.as_mut().unwrap() };

        // De-interleave.
        if TypeId::of::<F>() == TypeId::of::<Mono32>() {
            // Grab global state.
            let state = super::state();

            // Convert to speaker's native type.
            for (i, sample) in speakers.buffer.iter().cloned().enumerate() {
                state.l_buffer[i] = sample;
                state.r_buffer[i] = sample;
            }
        } else if TypeId::of::<F>() == TypeId::of::<Stereo32>() {
            // Grab global state.
            let state = super::state();

            // Convert to speaker's native type.
            for (i, sample) in speakers.buffer.chunks(2).enumerate() {
                state.l_buffer[i] = sample[0];
                state.r_buffer[i] = sample[1];
            }
        } else {
            unreachable!();
        }

        // Store 5.1 surround sample to resampler.
        let frame: Surround32 = self.1.frame().convert();
        speakers.resampler.0 = [
            frame.channels()[0],
            frame.channels()[1],
            frame.channels()[2],
            frame.channels()[3],
            frame.channels()[4],
            frame.channels()[5],
        ];
        // Store partial index from resampler.
        speakers.resampler.1 = self.1.index() % 1.0;
        // Unlock
        speakers.locked.store(false, SeqCst);
    }
}
