// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(unsafe_code)]

use std::{
    fmt::{Display, Error, Formatter},
    future::Future,
    marker::PhantomData,
    os::raw::c_void,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    task::{Context, Poll},
};

use fon::{chan::Ch32, Frame, Stream};

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
    pub(crate) sample_rate: Option<f64>,
    /// Microphone are locked
    locked: AtomicBool,
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
            locked: AtomicBool::new(false),
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
    fn set_channels<F>(&mut self) -> Option<bool>
    where
        F: Frame<Chan = Ch32>,
    {
        if F::CHAN_COUNT != self.channels.into() {
            if !matches!(F::CHAN_COUNT, 1 | 2 | 6) {
                panic!("Unknown speaker configuration")
            }
            self.channels = F::CHAN_COUNT as u8;
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

    pub(crate) fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<F> {
        // Change number of channels, if different than last call.
        self.set_channels::<F>()
            .expect("Microphone::record() called with invalid configuration");

        // Stream from microphone's buffer.
        MicrophoneStream(self, 0, PhantomData)
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

        // Safety
        if this.locked.load(SeqCst) {
            eprintln!("Tried to poll microphone before dropping stream");
            std::process::exit(1);
        }

        // If microphone is unconfigured, return Ready to configure and play.
        if this.channels == 0 {
            let _ = this.device.start();
            this.locked.store(true, SeqCst);
            return Poll::Ready(());
        }

        // Check if not woken, then yield.
        let mut pending = true;
        for fd in &this.device.fds {
            if !fd.should_yield() {
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
                for fd in &this.device.fds {
                    // Register waker
                    fd.register_waker(cx.waker());
                }
                // Not ready
                Poll::Pending
            }
            Ok(len) => {
                this.endi = len;
                // Ready, audio buffer has been filled!
                this.locked.store(true, SeqCst);
                Poll::Ready(())
            }
        }
    }
}

pub(crate) struct MicrophoneStream<F: Frame<Chan = Ch32>>(
    *mut Microphone,
    usize,
    PhantomData<F>,
);

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let mic = unsafe { self.0.as_mut().unwrap() };
        if self.1 >= mic.endi {
            return None;
        }
        let frame =
            F::from_channels(&mic.buffer[self.1 * mic.channels as usize..]);
        self.1 += 1;
        Some(frame)
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<F> {
    fn sample_rate(&self) -> Option<f64> {
        let mic = unsafe { self.0.as_mut().unwrap() };
        mic.sample_rate
    }

    fn len(&self) -> Option<usize> {
        let mic = unsafe { self.0.as_mut().unwrap() };
        Some(mic.endi)
    }
}

impl<F: Frame<Chan = Ch32>> Drop for MicrophoneStream<F> {
    fn drop(&mut self) {
        let mic = unsafe { self.0.as_mut().unwrap() };
        // Unlock
        mic.locked.store(false, SeqCst);
    }
}
