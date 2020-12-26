// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::{marker::PhantomData, task::{Poll, Context}, pin::Pin, future::Future};

use fon::{chan::{Channel, Ch32}, Frame, surround::Surround32, Resampler, Sink};

use super::{Pcm, AlsaPlayer, SndPcmStream, SndPcmState, pcm_hw_params, flush_buffer};

// FIXME?
use super::SoundDevice;

/// ALSA Speakers connection.
pub(crate) struct Speakers {
    /// FIXME: Remove in favor of new DL_API
    player: AlsaPlayer,
    /// ALSA PCM type for both speakers and microphones.
    pcm: Pcm,
    /// Index into audio frames to start writing.
    starti: usize,
    /// Raw buffer of audio yet to be played.
    buffer: Vec<Ch32>,
    /// Resampler context for speakers sink.
    resampler: ([Ch32; 6], f64),
    /// The number of frames in the buffer.
    period: u8,
    /// Number of available channels
    pub(crate) channels: u8,
    /// The sample rate of the speakers.
    pub(crate) sample_rate: Option<f64>,
}

impl Speakers {
    /// Attempt to connect to a speaker by id.
    pub(crate) fn connect(id: &crate::SpeakersId) -> Option<Self> {
        // Load Player ALSA module
        let player = AlsaPlayer::new()?;
        // Create Playback PCM.
        let pcm = Pcm::new(id.0.desc(), SndPcmStream::Playback)?;
        Some(Self {
            player,
            pcm,
            starti: 0,
            buffer: Vec::new(),
            sample_rate: None,
            channels: 0,
            resampler: ([Ch32::MID; 6], 0.0),
            period: 0,
        })
    }

    /// Attempt to configure the speaker for a specific number of channels.
    pub(crate) fn set_channels<F>(&mut self) -> Option<bool>
        where F: Frame<Chan = Ch32>
    {
        if F::CHAN_COUNT != self.channels.into() {
            if !matches!(F::CHAN_COUNT, 1 | 2 | 6) {
                panic!("Unknown speaker configuration")
            }
            self.channels = F::CHAN_COUNT as u8;
            // Configure Hardware Parameters
            pcm_hw_params(
                &self.pcm.sound_device,
                self.channels,
                &mut self.buffer,
                &mut self.sample_rate,
                &mut self.period,
            )?;
            Some(true)
        } else {
            Some(false)
        }
    }

    /// Generate an audio sink for the user to fill.
    pub(crate) fn play<F>(&mut self) -> SpeakersSink<'_, F>
        where F: Frame<Chan = Ch32>
    {
        // Change number of channels, if different than last call.
        if self.set_channels::<F>()
            .expect("Speaker::play() called with invalid configuration")
        {
            flush_buffer(self.pcm.sound_device.0);
        }
        // Convert the resampler to the target speaker configuration.
        let resampler = Resampler::<F>::new(
            Surround32::from_channels(&self.resampler.0[..]).convert(),
            self.resampler.1,
        );
        // Create a sink that borrows this speaker's buffer mutably.
        SpeakersSink(self, resampler, PhantomData)
    }
}

impl Future for &mut Speakers {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to speakers.
        let this = self.get_mut();

        // Check if speakers have been used yet, if not return Ready.
        if this.channels == 0 {
            return Poll::Ready(());
        }

        // Attempt to write remaining internal speaker buffer to the speakers.
        let result = unsafe {
            this.player.snd_pcm_writei(
                &this.pcm.sound_device,
                this.buffer.as_ptr().cast(),
                this.period.into(),
            )
        };

        // Check if it succeeds, then return Ready.
        match result {
            Err(error) => {
                match error {
                    // Edge-triggered epoll should only go into pending mode if
                    // read/write call results in EAGAIN (according to epoll man
                    // page)
                    -11 => { /* Pending */ }
                    -32 => match this
                        .pcm
                        .device
                        .snd_pcm_state(&this.pcm.sound_device)
                    {
                        SndPcmState::Xrun => {
                            // Player samples are not generated fast enough
                            this.pcm
                                .device
                                .snd_pcm_prepare(&this.pcm.sound_device)
                                .unwrap();
                            unsafe {
                                this.player
                                    .snd_pcm_writei(
                                        &this.pcm.sound_device,
                                        this.buffer.as_ptr().cast(),
                                        this.period.into(),
                                    )
                                    .unwrap();
                            }
                        }
                        st => {
                            eprintln!(
                                "Incorrect state = {:?} (XRUN): Report Bug to \
                                 https://github.com/libcala/wavy/issues/new",
                                st
                            );
                            unreachable!()
                        }
                    },
                    -77 => {
                        eprintln!(
                            "Incorrect state (-EBADFD): Report Bug to \
                             https://github.com/libcala/wavy/issues/new"
                        );
                        unreachable!()
                    }
                    -86 => {
                        eprintln!(
                            "Stream got suspended, trying to recover… \
                            (-ESTRPIPE)"
                        );
                        if this
                            .pcm
                            .device
                            .snd_pcm_resume(&this.pcm.sound_device)
                            .is_ok()
                        {
                            // Prepare, so we keep getting samples.
                            this.pcm
                                .device
                                .snd_pcm_prepare(&this.pcm.sound_device)
                                .unwrap();
                            unsafe {
                                this.player
                                    .snd_pcm_writei(
                                        &this.pcm.sound_device,
                                        this.buffer.as_ptr().cast(),
                                        this.period.into(),
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                // Register waker, and then return not ready.
                this.pcm.fd.register_waker(cx.waker());
                Poll::Pending
            }
            Ok(len) => {
                // Shift buffer.
                this.buffer.drain(..len as usize * this.channels as usize);
                this.starti = this.buffer.len() / this.channels as usize;
                this.buffer.resize(
                    this.period as usize * this.channels as usize,
                    Ch32::MID,
                );
                // Ready for more samples.
                Poll::Ready(())
            }
        }
    }
}

pub(crate) struct SpeakersSink<'a, F: Frame<Chan = Ch32>>(
    &'a mut Speakers,
    Resampler<F>,
    PhantomData<F>,
);

impl<F: Frame<Chan = Ch32>> Sink<F> for SpeakersSink<'_, F> {
    fn sample_rate(&self) -> f64 {
        self.0.sample_rate.unwrap()
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.1
    }

    #[allow(unsafe_code)]
    fn buffer(&mut self) -> &mut [F] {
        let data = self.0.buffer.as_mut_ptr().cast();
        let count = self.0.period.into();
        unsafe {
            &mut std::slice::from_raw_parts_mut(data, count)[self.0.starti..]
        }
    }
}

impl<F: Frame<Chan = Ch32>> Drop for SpeakersSink<'_, F> {
    fn drop(&mut self) {
        // Store 5.1 surround sample to resampler.
        let frame: Surround32 = self.1.frame().convert();
        self.0.resampler.0 = [
            frame.channels()[0],
            frame.channels()[1],
            frame.channels()[2],
            frame.channels()[3],
            frame.channels()[4],
            frame.channels()[5],
        ];
        // Store partial index from resampler.
        self.0.resampler.1 = self.1.index() % 1.0;
    }
}
