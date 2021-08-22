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

/// Open a PCM Device.
pub(crate) fn open(
    name: *const c_char,
    stream: SndPcmStream,
) -> Option<(*mut c_void, *mut c_void, u8)> {
    unsafe {
        let pcm = pcm::open(name, stream, SndPcmMode::Nonblock).ok()?;
        let hwp = pcm::hw_params_malloc().ok()?;
        let mut channels = 0;
        reset_hwp(pcm, hwp)?;
        for i in 1..=8 {
            if pcm::hw_test_channels(pcm, hwp, i).is_ok() {
                channels |= 1 << (i - 1);
            }
        }
        Some((pcm, hwp, channels))
    }
}

pub(crate) trait SoundDevice:
    std::fmt::Display + From<AudioDevice>
{
    const INPUT: bool;

    fn pcm(&self) -> *mut c_void;
    fn hwp(&self) -> *mut c_void;
}

/// An Audio Device (input or output).
#[derive(Debug)]
pub(crate) struct AudioDevice {
    /// Human-readable name for the device.
    pub(crate) name: String,
    /// PCM For Device.
    pub(crate) pcm: *mut c_void,
    /// Hardware parameters for device.
    pub(crate) hwp: *mut c_void,
    /// Bitflags for numbers of channels (which of 1-8 are supported)
    pub(crate) supported: u8,
    /// File descriptors associated with this device.
    pub(crate) fds: Vec<smelling_salts::Device>,
}

impl AudioDevice {
    /// Generate file descriptors.
    pub(crate) fn start(&mut self) -> Option<()> {
        assert!(self.fds.is_empty());
        // Get file descriptor.
        let fd_list = unsafe { pcm::poll_descriptors(self.pcm).ok()? };
        // Add to list.
        for fd in fd_list {
            self.fds.push(smelling_salts::Device::new(fd.fd, unsafe {
                smelling_salts::Watcher::from_raw(fd.events as u32)
            }));
        }
        Some(())
    }
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        // Unregister async file descriptors before closing the PCM.
        for fd in &mut self.fds {
            fd.stop();
        }
        // Free hardware parameters and close PCM
        unsafe {
            pcm::hw_params_free(self.hwp);
            pcm::close(self.pcm).unwrap();
        }
    }
}

/// Return a list of available audio devices.
pub(crate) fn device_list<D: SoundDevice, F: Fn(D) -> T, T>(
    abstrakt: F,
) -> Vec<T> {
    super::ALSA.with(|alsa| {
        if let Some(alsa) = alsa {
            device_list_internal(&alsa, abstrakt)
        } else {
            Vec::new()
        }
    })
}

fn device_list_internal<D: SoundDevice, F: Fn(D) -> T, T>(
    alsa: &Alsa,
    abstrakt: F,
) -> Vec<T> {
    let tpcm = CStr::from_bytes_with_nul(b"pcm\0").unwrap();
    let tname = CStr::from_bytes_with_nul(b"NAME\0").unwrap();
    let tdesc = CStr::from_bytes_with_nul(b"DESC\0").unwrap();
    let tioid = CStr::from_bytes_with_nul(b"IOID\0").unwrap();

    let mut hints = MaybeUninit::uninit();
    let mut devices = Vec::new();
    unsafe {
        if (alsa.snd_device_name_hint)(-1, tpcm.as_ptr(), hints.as_mut_ptr())
            < 0
        {
            return Vec::new();
        }
        let hints = hints.assume_init();
        let mut n = hints;
        while !(*n).is_null() {
            // Allocate 3 C Strings describing device.
            let pcm_name = (alsa.snd_device_name_get_hint)(*n, tname.as_ptr());
            let io = (alsa.snd_device_name_get_hint)(*n, tioid.as_ptr());
            debug_assert_ne!(pcm_name, std::ptr::null_mut());

            // Convert description to Rust String
            let name = match CStr::from_ptr(pcm_name).to_str() {
                Ok(x) if x.starts_with("sysdefault") => {
                    n = n.offset(1);
                    continue;
                }
                Ok("null") => {
                    // Can't use epoll on null.
                    n = n.offset(1);
                    continue;
                }
                Ok("default") => "Default".to_string(),
                _a => {
                    let name =
                        (alsa.snd_device_name_get_hint)(*n, tdesc.as_ptr());
                    assert_ne!(name, std::ptr::null_mut());
                    let rust =
                        CStr::from_ptr(name).to_string_lossy().to_string();
                    free(name.cast());
                    rust.replace("\n", ": ")
                }
            };

            // Check device io direction.
            let is_input = io.is_null() || *(io.cast::<u8>()) == b'I';
            let is_output = io.is_null() || *(io.cast::<u8>()) == b'O';
            if !io.is_null() {
                free(io.cast());
            }

            // Right input type?
            if (D::INPUT && is_input) || (!D::INPUT && is_output) {
                // Try to connect to PCM.
                let dev = open(
                    pcm_name,
                    if D::INPUT {
                        SndPcmStream::Capture
                    } else {
                        SndPcmStream::Playback
                    },
                );

                if let Some((pcm, hwp, supported)) = dev {
                    // Add device to list of devices.
                    devices.push(abstrakt(D::from(AudioDevice {
                        name,
                        pcm,
                        hwp,
                        supported,
                        fds: Vec::new(),
                    })));
                }
            }
            free(pcm_name.cast());
            n = n.offset(1);
        }
        (alsa.snd_device_name_free_hint)(hints);
    }
    devices
}

#[allow(unsafe_code)]
pub(crate) fn pcm_hw_params(
    device: &AudioDevice,
    channels: u8,
    buffer: &mut Vec<Ch32>,
    sample_rate: &mut Option<u32>,
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
