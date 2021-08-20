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

use fon::{chan::Ch32, Frame};

use super::{
    asound, pcm_hw_params, AudioDevice, SndPcmState, SndPcmStream, SoundDevice,
    DEFAULT,
};

pub(crate) struct Microphone {
    // PCM I/O Handle
    device: AudioDevice,
    // Interleaved Audio Buffer.
    buffer: Vec<Ch32>,
    // The period of the microphone.
    period: u16,
    // Index to stop reading.
    endi: usize,
    // Number of channels on the Microphone.
    pub(crate) channels: u8,
    // Sample Rate of The Microphone (src)
    pub(crate) sample_rate: Option<u32>,
}

impl SoundDevice for Microphone {
    const INPUT: bool = true;

    fn pcm(&self) -> *mut c_void {
        self.device.pcm
    }

    fn hwp(&self) -> *mut c_void {
        self.device.pcm
    }
}

impl From<AudioDevice> for Microphone {
    fn from(device: AudioDevice) -> Self {
        Self {
            device,
            buffer: Vec::new(),
            period: 0,
            channels: 0,
            endi: 0,
            sample_rate: None,
        }
    }
}

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.device.name.as_str())
    }
}

impl Default for Microphone {
    fn default() -> Self {
        let (pcm, hwp, supported) =
            super::open(DEFAULT.as_ptr().cast(), SndPcmStream::Capture)
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

impl Microphone {
    /// Attempt to configure the microphone for a specific number of channels.
    fn set_channels<const CH: usize>(&mut self) -> Option<bool> {
        if CH != self.channels.into() {
            if !matches!(CH, 1 | 2 | 6) {
                panic!("Unknown speaker configuration")
            }
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

    pub(crate) fn record<const CH: usize>(
        &mut self,
    ) -> MicrophoneStream<'_, CH> {
        // Change number of channels, if different than last call.
        self.set_channels::<CH>()
            .expect("Microphone::record() called with invalid configuration");

        // Stream from microphone's buffer.
        MicrophoneStream(self, 0)
    }

    pub(crate) fn channels(&self) -> u8 {
        self.device.supported
    }
}

impl Future for Microphone {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to microphone.
        let this = self.get_mut();

        // If microphone is unconfigured, return Ready to configure and play.
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

        // Attempt to overwrite the internal microphone buffer.
        let result = unsafe {
            asound::pcm::readi(
                this.device.pcm,
                this.buffer.as_mut_slice().as_mut_ptr(),
                this.period,
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
                    -77 => {
                        eprintln!(
                            "Incorrect state (-EBADFD): Report Bug to \
                        https://github.com/libcala/wavy/issues/new"
                        );
                        unreachable!()
                    }
                    -32 => {
                        match unsafe { asound::pcm::state(this.device.pcm) } {
                            SndPcmState::Xrun => {
                                eprintln!("Microphone XRUN: Latency cause?");
                                unsafe {
                                    asound::pcm::prepare(this.device.pcm)
                                        .unwrap();
                                }
                            }
                            st => {
                                eprintln!(
                                "Incorrect state = {:?} (XRUN): Report Bug \
                            to https://github.com/libcala/wavy/issues/new",
                                st
                            );
                                unreachable!()
                            }
                        }
                    }
                    -86 => {
                        eprintln!(
                        "Stream got suspended, trying to recover… (-ESTRPIPE)"
                    );
                        unsafe {
                            if asound::pcm::resume(this.device.pcm).is_ok() {
                                // Prepare, so we keep getting samples.
                                asound::pcm::prepare(this.device.pcm).unwrap();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                // Not Ready
                let mut pending = Poll::Pending;
                for fd in &mut this.device.fds {
                    // Register waker
                    pending = fd.sleep(cx);
                }
                pending
            }
            Ok(len) => {
                this.endi = len;
                // Ready, audio buffer has been filled!
                Poll::Ready(())
            }
        }
    }
}

pub(crate) struct MicrophoneStream<'a, const CH: usize>(
    &'a mut Microphone,
    usize,
);

impl<const CH: usize> Iterator for MicrophoneStream<'_, CH> {
    type Item = Frame<Ch32, CH>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0.endi {
            return None;
        }
        let mut frame = Self::Item::default();
        for (i, chan) in frame.channels_mut().iter_mut().enumerate() {
            *chan = self.0.buffer[self.1 * self.0.channels as usize + i];
        }
        self.1 += 1;
        Some(frame)
    }
}

impl<const CH: usize> MicrophoneStream<'_, CH> {
    pub(crate) fn sample_rate(&self) -> Option<u32> {
        self.0.sample_rate
    }

    pub(crate) fn len(&self) -> Option<usize> {
        Some(self.0.endi)
    }
}
