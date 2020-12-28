// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::{
    convert::TryInto,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use fon::{
    chan::{Ch32, Channel},
    Frame, Stream,
};

// ALSA bindings.
mod asound;
//
mod speakers;

// Implementation Expectations:
pub(crate) use asound::{
    device_list::{device_list, AudioDst, AudioSrc, reset_hwp},
    PollFd, SndPcmAccess, SndPcmFormat, SndPcmMode, SndPcmState, SndPcmStream,
};
pub(crate) use speakers::{Speakers, SpeakersSink};

#[allow(unsafe_code)]
fn flush_buffer(pcm: *mut std::os::raw::c_void) {
    unsafe {
        // Empty the audio buffer to avoid artifacts on startup.
        let _ = asound::pcm::drop(pcm);
        // Once it's empty, it needs to be re-prepared.
        let _ = asound::pcm::prepare(pcm);
    }
}

#[allow(unsafe_code)]
fn pcm_hw_params(
    pcm: &Pcm,
    channels: u8,
    buffer: &mut Vec<Ch32>,
    sample_rate: &mut Option<f64>,
    period: &mut u8,
) -> Option<()> {
    unsafe {
        // Reset hardware parameters to any interleaved native endian float32
        reset_hwp(pcm.dev.pcm, pcm.dev.hwp)?;

        // Set Hz near library target Hz.
        asound::pcm::hw_params_set_rate_near(
            pcm.dev.pcm,
            pcm.dev.hwp,
            &mut crate::consts::SAMPLE_RATE.into(),
            &mut 0,
        )
        .ok()?;
        // Set the number of channels.
        asound::pcm::hw_set_channels(pcm.dev.pcm, pcm.dev.hwp, channels).ok()?;
        // Set period near library target period.
        let mut period_size = crate::consts::PERIOD.into();
        asound::pcm::hw_params_set_period_size_near(
            pcm.dev.pcm,
            pcm.dev.hwp,
            &mut period_size,
            &mut 0,
        )
        .ok()?;
        // Some buffer size should always be available (match period).
        asound::pcm::hw_params_set_buffer_size_near(
            pcm.dev.pcm,
            pcm.dev.hwp,
            &mut period_size,
        )
        .ok()?;
        // Should always be able to apply parameters that succeeded
        asound::pcm::hw_params(pcm.dev.pcm, pcm.dev.hwp).ok()?;
        // Now that a configuration has been chosen, we can retreive the actual
        // exact sample rate.
        *sample_rate = Some(asound::pcm::hw_get_rate(pcm.dev.hwp)?);

        // Set the period of the buffer.
        *period = period_size.try_into().ok()?;

        // Resize the buffer
        buffer.resize(*period as usize * channels as usize, Ch32::MID);
    }

    Some(())
}

// Speakers/Microphone Shared Code for ALSA.
pub(super) struct Pcm {
    dev: asound::device_list::AudioDevice,
    fd: smelling_salts::Device,
}

impl Pcm {
    /// Create a new async PCM.  If it fails return `None`.
    #[allow(unsafe_code)]
    fn new(
        dev: asound::device_list::AudioDevice,
    ) -> Option<Self> {
        // Get file descriptor
        let fd_list =
            unsafe { asound::pcm::poll_descriptors(dev.pcm).ok()? };
        // FIXME: More?
        assert_eq!(fd_list.len(), 1);
        // Register file descriptor with OS's I/O Event Notifier
        let fd = smelling_salts::Device::new(
            fd_list[0].fd,
            #[allow(unsafe_code)]
            unsafe {
                smelling_salts::Watcher::from_raw(fd_list[0].events as u32)
            },
        );

        Some(Pcm { dev, fd })
    }
}

impl Drop for Pcm {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // Unregister async file descriptor before closing the PCM.
        self.fd.old();
    }
}

pub(crate) struct Microphone {
    // PCM I/O Handle
    pcm: Pcm,
    // Interleaved Audio Buffer.
    buffer: Vec<Ch32>,
    // The period of the microphone.
    period: u8,
    // Number of channels on the Microphone.
    pub(crate) channels: u8,
    // Sample Rate of The Microphone (src)
    pub(crate) sample_rate: Option<f64>,
}

impl Microphone {
    pub(crate) fn new(id: crate::MicrophoneId) -> Option<Self> {
        // Create Capture PCM.
        let pcm = Pcm::new(id.0.0)?;
        // Return successfully
        Some(Self {
            pcm,
            buffer: Vec::new(),
            period: 0,
            channels: 0,
            sample_rate: None,
        })
    }

    /// Attempt to configure the microphone for a specific number of channels.
    pub(crate) fn set_channels<F>(&mut self) -> Option<bool>
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
                &self.pcm,
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
    ) -> MicrophoneStream<'_, F> {
        // Stream from microphone's buffer.
        MicrophoneStream(self, 0, PhantomData)
    }
}

impl Future for Microphone {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to microphone.
        let this = self.get_mut();

        // Attempt to overwrite the internal microphone buffer.
        let result = unsafe {
            asound::pcm::readi(
                this.pcm.dev.pcm,
                this.buffer.as_mut_slice().as_mut_ptr(),
                this.period,
            )
        };

        // Check if it succeeds, then return Ready.
        if let Err(error) = result {
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
                    match unsafe { asound::pcm::state(this.pcm.dev.pcm) } {
                        SndPcmState::Xrun => {
                            eprintln!("Microphone XRUN: Latency cause?");
                            unsafe {
                                asound::pcm::prepare(this.pcm.dev.pcm)
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
                        if asound::pcm::resume(this.pcm.dev.pcm).is_ok() {
                            // Prepare, so we keep getting samples.
                            asound::pcm::prepare(this.pcm.dev.pcm)
                                .unwrap();
                        }
                    }
                }
                _ => unreachable!(),
            }
            // Register waker
            this.pcm.fd.register_waker(cx.waker());
            // Not ready
            Poll::Pending
        } else {
            // Ready, audio buffer has been filled!
            Poll::Ready(())
        }
    }
}

pub(crate) struct MicrophoneStream<'a, F: Frame<Chan = Ch32>>(
    &'a mut Microphone,
    usize,
    PhantomData<F>,
);

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0.period.into() {
            return None;
        }
        Some(F::from_channels(
            &self.0.buffer[self.1 * self.0.channels as usize..],
        ))
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<'_, F> {
    fn sample_rate(&self) -> Option<f64> {
        self.0.sample_rate
    }

    fn len(&self) -> Option<usize> {
        Some(self.0.period.into())
    }
}
