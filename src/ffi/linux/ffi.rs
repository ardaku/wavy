// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::convert::TryInto;

use fon::chan::{Ch32, Channel};

mod asound;
mod microphone;
mod speakers;

// Implementation Expectations:
pub(crate) use asound::{
    device_list::{device_list, reset_hwp, AudioDst, AudioSrc},
    PollFd, SndPcmAccess, SndPcmFormat, SndPcmMode, SndPcmState, SndPcmStream,
};
pub(crate) use microphone::{Microphone, MicrophoneStream};
pub(crate) use speakers::{Speakers, SpeakersSink};

#[allow(unsafe_code)]
fn pcm_hw_params(
    pcm: &Pcm,
    channels: u8,
    buffer: &mut Vec<Ch32>,
    sample_rate: &mut Option<f64>,
    period: &mut u16,
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
        asound::pcm::hw_set_channels(pcm.dev.pcm, pcm.dev.hwp, channels)
            .ok()?;
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

        // Empty the audio buffer to avoid artifacts on startup.
        let _ = asound::pcm::drop(pcm.dev.pcm);
        // Should always be able to apply parameters that succeeded
        asound::pcm::prepare(pcm.dev.pcm).ok()?;
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
    fn new(dev: asound::device_list::AudioDevice) -> Option<Self> {
        // Get file descriptor
        let fd_list = unsafe { asound::pcm::poll_descriptors(dev.pcm).ok()? };
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
