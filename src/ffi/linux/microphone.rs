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

struct MicrophoneInner {
    // PCM I/O Handle
    device: AudioDevice,
    // Interleaved Audio Buffer.
    buffer: Vec<Ch32>,
    // The period of the microphone.
    period: u16,
    // Index to stop reading.
    endi: usize,
    /// Microphone are locked
    locked: AtomicBool,
}

pub(crate) struct Microphone {
    // Number of channels on the Microphone.
    pub(crate) channels: u8,
    // Sample Rate of The Microphone (src)
    pub(crate) sample_rate: Option<f64>,
    /// Leaked shared box
    inner: *mut MicrophoneInner,
}

impl Drop for Microphone {
    fn drop(&mut self) {
        // Safety
        if unsafe { (*self.inner).locked.load(SeqCst) } {
            eprintln!("Microphone dropped before dropping stream");
            std::process::exit(1);
        }

        unsafe { drop(Box::from_raw(self.inner)) };
    }
}

impl SoundDevice for Microphone {
    const INPUT: bool = true;

    fn pcm(&self) -> *mut c_void {
        unsafe { (*self.inner).device.pcm }
    }

    fn hwp(&self) -> *mut c_void {
        unsafe { (*self.inner).device.pcm }
    }
}

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // Safety
        if unsafe { (*self.inner).locked.load(SeqCst) } {
            eprintln!("Tried to display speakers before dropping sink");
            std::process::exit(1);
        }

        unsafe { f.write_str((*self.inner).device.name.as_str()) }
    }
}

impl From<AudioDevice> for Microphone {
    fn from(device: AudioDevice) -> Self {
        Self {
            channels: 0,
            sample_rate: None,
            inner: Box::leak(Box::new(MicrophoneInner {
                device,
                buffer: Vec::new(),
                period: 0,
                endi: 0,
                locked: AtomicBool::new(false),
            })),
        }
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
    fn set_channels<F>(&mut self, inner: &mut MicrophoneInner) -> Option<bool>
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
                &inner.device,
                self.channels,
                &mut inner.buffer,
                &mut self.sample_rate,
                &mut inner.period,
            )?;
            Some(true)
        } else {
            Some(false)
        }
    }

    pub(crate) fn record<F: Frame<Chan = Ch32>>(
        &mut self,
    ) -> MicrophoneStream<F> {
        // Always called after ready, so should be safe
        let inner = unsafe { self.inner.as_mut().unwrap() };

        // Change number of channels, if different than last call.
        self.set_channels::<F>(inner)
            .expect("Microphone::record() called with invalid configuration");

        // Stream from microphone's buffer.
        MicrophoneStream(inner, 0, PhantomData, self.sample_rate, self.channels)
    }

    pub(crate) fn channels(&self) -> u8 {
        // Safety
        if unsafe { (*self.inner).locked.load(SeqCst) } {
            eprintln!("Tried to poll speakers before dropping sink");
            std::process::exit(1);
        }

        unsafe { (*self.inner).device.supported }
    }
}

impl Future for Microphone {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to microphone.
        let this = self.get_mut();

        // Safety
        if unsafe { (*this.inner).locked.load(SeqCst) } {
            eprintln!("Tried to poll microphone before dropping stream");
            std::process::exit(1);
        }
        //
        let inner = unsafe { this.inner.as_mut().unwrap() };

        // If microphone is unconfigured, return Ready to configure and play.
        if this.channels == 0 {
            let _ = inner.device.start();
            inner.locked.store(true, SeqCst);
            return Poll::Ready(());
        }

        // Check if not woken, then yield.
        let mut pending = true;
        for fd in &inner.device.fds {
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
                inner.device.pcm,
                inner.buffer.as_mut_slice().as_mut_ptr(),
                inner.period,
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
                        match unsafe { asound::pcm::state(inner.device.pcm) } {
                            SndPcmState::Xrun => {
                                eprintln!("Microphone XRUN: Latency cause?");
                                unsafe {
                                    asound::pcm::prepare(inner.device.pcm)
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
                            if asound::pcm::resume(inner.device.pcm).is_ok() {
                                // Prepare, so we keep getting samples.
                                asound::pcm::prepare(inner.device.pcm).unwrap();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                for fd in &inner.device.fds {
                    // Register waker
                    fd.register_waker(cx.waker());
                }
                // Not ready
                Poll::Pending
            }
            Ok(len) => {
                inner.endi = len;
                // Ready, audio buffer has been filled!
                inner.locked.store(true, SeqCst);
                Poll::Ready(())
            }
        }
    }
}

pub(crate) struct MicrophoneStream<F: Frame<Chan = Ch32>>(
    *mut MicrophoneInner,
    usize,
    PhantomData<F>,
    Option<f64>,
    u8,
);

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let mic = unsafe { self.0.as_mut().unwrap() };
        if self.1 >= mic.endi {
            return None;
        }
        let frame = F::from_channels(&mic.buffer[self.1 * self.4 as usize..]);
        self.1 += 1;
        Some(frame)
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<F> {
    fn sample_rate(&self) -> Option<f64> {
        self.3
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
