// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(unsafe_code)]

use std::{
    fmt::{Display, Error, Formatter},
    future::Future,
    os::raw::c_void,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{
    chan::{Ch32, Channel},
    pos::{BackL, BackR, Front, FrontL, FrontR, Left, Lfe, Right},
    Frame,
};

use super::{
    asound, pcm_hw_params, AudioDevice, SndPcmState, SndPcmStream, SoundDevice,
    DEFAULT,
};

/// ALSA Speakers connection.
pub(crate) struct Speakers {
    /// ALSA PCM type for both speakers and microphones.
    device: AudioDevice,
    /// Index into audio frames to start writing.
    starti: usize,
    /// Raw buffer of audio yet to be played.
    buffer: Vec<Ch32>,
    /// The number of frames in the buffer.
    period: u16,
    /// Number of available channels
    pub(crate) channels: u8,
    /// The sample rate of the speakers.
    pub(crate) sample_rate: Option<u32>,
}

impl SoundDevice for Speakers {
    const INPUT: bool = false;

    fn pcm(&self) -> *mut c_void {
        self.device.pcm
    }

    fn hwp(&self) -> *mut c_void {
        self.device.pcm
    }
}

impl Display for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.device.name.as_str())
    }
}

impl From<AudioDevice> for Speakers {
    fn from(device: AudioDevice) -> Self {
        Self {
            device,
            starti: 0,
            buffer: Vec::new(),
            sample_rate: None,
            channels: 0,
            period: 0,
        }
    }
}

impl Default for Speakers {
    fn default() -> Self {
        let (pcm, hwp, supported) =
            super::open(DEFAULT.as_ptr().cast(), SndPcmStream::Playback)
                .unwrap();
        Self::from(AudioDevice {
            name: "Default".to_string(),
            pcm,
            hwp,
            supported,
            fds: Vec::new(),
        })
    }
}

impl Speakers {
    /// Attempt to configure the speaker for a specific number of channels.
    fn set_channels<const CH: usize>(&mut self) -> Option<bool> {
        if CH != self.channels.into() {
            self.channels = CH as u8;
            // Configure Hardware Parameters
            pcm_hw_params(
                &self.device,
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
    pub(crate) fn play<const CH: usize>(&mut self) -> SpeakersSink<CH> {
        // Change number of channels, if different than last call.
        self.set_channels::<CH>()
            .expect("Speaker::play() called with invalid configuration");
        // Create a sink that borrows this speaker's buffer mutably.
        SpeakersSink(self)
    }

    pub(crate) fn channels(&self) -> u8 {
        self.device.supported
    }
}

impl Future for &mut Speakers {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to speakers.
        let this = self.get_mut();

        // If speaker is unconfigured, return Ready to configure and play.
        if this.channels == 0 {
            let _ = this.device.start();
            return Poll::Ready(());
        }

        // Check if not woken, then yield.
        let mut pending = true;
        for fd in &this.device.fds {
            if !fd.pending() {
                pending = false;
                break;
            }
        }

        if pending {
            return Poll::Pending;
        }

        // Attempt to write remaining internal speaker buffer to the speakers.
        let result = unsafe {
            asound::pcm::writei(
                this.device.pcm,
                this.buffer.as_ptr(),
                this.period.into(),
            )
        };

        // Check if it succeeds, then return Ready.
        let len = match result {
            Ok(len) => len,
            Err(error) => {
                match error {
                    // Edge-triggered epoll should only go into pending mode if
                    // read/write call results in EAGAIN (according to epoll man
                    // page)
                    -11 => {
                        let mut pending = Poll::Pending;
                        for fd in &mut this.device.fds {
                            // Register waker, and then return not ready.
                            pending = fd.sleep(cx);
                        }
                        return pending;
                    }
                    -32 => {
                        match unsafe { asound::pcm::state(this.device.pcm) } {
                            SndPcmState::Xrun => {
                                // Player samples are not generated fast enough
                                unsafe {
                                    asound::pcm::prepare(this.device.pcm)
                                        .unwrap();
                                    asound::pcm::writei(
                                        this.device.pcm,
                                        this.buffer.as_ptr(),
                                        this.period.into(),
                                    )
                                    .unwrap()
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
                        }
                    }
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

                        // Prepare, so we keep getting samples.
                        unsafe {
                            // Whether this works or not, we want to prepare.
                            let _ = asound::pcm::resume(this.device.pcm);
                            // Prepare
                            asound::pcm::prepare(this.device.pcm).unwrap();
                            asound::pcm::writei(
                                this.device.pcm,
                                this.buffer.as_ptr(),
                                this.period.into(),
                            )
                            .unwrap()
                        }
                    }
                    _ => unreachable!(),
                }
            }
        };

        // Shift buffer.
        this.buffer.drain(..len * this.channels as usize);
        this.starti = this.buffer.len() / this.channels as usize;
        this.buffer
            .resize(this.period as usize * this.channels as usize, Ch32::MID);
        // Ready for more samples.
        Poll::Ready(())
    }
}

pub(crate) struct SpeakersSink<const CH: usize>(*mut Speakers);

impl<const CH: usize> SpeakersSink<CH> {
    pub(crate) fn sample_rate(&self) -> u32 {
        unsafe { (*self.0).sample_rate.unwrap() }
    }

    pub(crate) fn len(&self) -> usize {
        unsafe { (*self.0).period.into() }
    }

    pub(crate) fn sink<I: Iterator<Item = Frame<Ch32, 8>>>(&mut self, iter: I) {
        // Swap from SMPTE 7.1 to Linux 7.1 Channel Order:
        //  0. Front Left
        //  1. Front Right
        //  2. Surround Left
        //  3. Surround Right
        //  4. Front Center
        //  5. LFE
        //  6. Side Left
        //  7. Side Right
        for (channels, frame) in unsafe {
            (*self.0)
                .buffer
                .chunks_mut(8)
                .skip((*self.0).starti)
                .zip(iter)
        } {
            channels[0] = frame[FrontL];
            channels[1] = frame[FrontR];
            channels[2] = frame[Left];
            channels[3] = frame[Right];
            channels[4] = frame[Front];
            channels[5] = frame[Lfe];
            channels[6] = frame[BackL];
            channels[7] = frame[BackR];
        }
    }
}
