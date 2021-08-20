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
    convert::TryInto,
    ffi::CStr,
    mem::MaybeUninit,
    os::raw::{c_char, c_void},
};

use fon::chan::{Ch32, Channel};

use super::{
    free, pcm, Alsa, SndPcmAccess, SndPcmFormat, SndPcmMode, SndPcmStream,
};

pub(crate) const DEFAULT: &[u8] = b"default\0";

/// Reset hardware parameters.
pub(crate) unsafe fn reset_hwp(
    pcm: *mut c_void,
    hwp: *mut c_void,
) -> Option<()> {
    let format = if cfg!(target_endian = "little") {
        SndPcmFormat::FloatLe
    } else if cfg!(target_endian = "big") {
        SndPcmFormat::FloatBe
    } else {
        unreachable!()
    };
    pcm::hw_params_any(pcm, hwp).ok()?;
    pcm::hw_params_set_access(pcm, hwp, SndPcmAccess::RwInterleaved).ok()?;
    pcm::hw_params_set_format(pcm, hwp, format).ok()?;
    Some(())
}

pub(crate) trait SoundDevice:
    std::fmt::Display + From<AudioDevice>
{
    const INPUT: bool;

    fn pcm(&self) -> *mut c_void;
    fn hwp(&self) -> *mut c_void;
}

#[allow(unsafe_code)]
pub(crate) fn pcm_hw_params(
    device: &AudioDevice,
    channels: u8,
    buffer: &mut Vec<Ch32>,
    sample_rate: &mut Option<f64>,
    period: &mut u16,
) -> Option<()> {
    unsafe {
        // Reset hardware parameters to any interleaved native endian float32
        reset_hwp(device.pcm, device.hwp)?;

        // Set Hz near library target Hz.
        pcm::hw_params_set_rate_near(
            device.pcm,
            device.hwp,
            &mut crate::consts::SAMPLE_RATE.into(),
            &mut 0,
        )
        .ok()?;
        // Set the number of channels.
        pcm::hw_set_channels(device.pcm, device.hwp, channels).ok()?;
        // Set period near library target period.
        let mut period_size = crate::consts::PERIOD.into();
        pcm::hw_params_set_period_size_near(
            device.pcm,
            device.hwp,
            &mut period_size,
            &mut 0,
        )
        .ok()?;
        // Some buffer size should always be available (match period).
        pcm::hw_params_set_buffer_size_near(
            device.pcm,
            device.hwp,
            &mut period_size,
        )
        .ok()?;
        // Should always be able to apply parameters that succeeded
        pcm::hw_params(device.pcm, device.hwp).ok()?;

        // Now that a configuration has been chosen, we can retreive the actual
        // exact sample rate.
        *sample_rate = Some(pcm::hw_get_rate(device.hwp)?);

        // Set the period of the buffer.
        *period = period_size.try_into().ok()?;

        // Resize the buffer
        buffer.resize(*period as usize * channels as usize, Ch32::MID);

        // Empty the audio buffer to avoid artifacts on startup.
        let _ = pcm::drop(device.pcm);
        // Should always be able to apply parameters that succeeded
        pcm::prepare(device.pcm).ok()?;
    }

    Some(())
}
