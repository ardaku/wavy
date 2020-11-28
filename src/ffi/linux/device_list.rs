// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(unsafe_code)]

use std::{
    mem::MaybeUninit, os::raw::{c_char}, ffi::CStr, fmt::{Formatter, Display, Error}
};
use super::{free, Alsa};

pub(crate) trait SoundDevice: Display + From<AudioDevice> {
    const INPUT: bool;

    fn desc(&self) -> *mut c_char;
}

#[derive(Debug)]
pub(crate) struct AudioSrc(AudioDevice);

impl SoundDevice for AudioSrc {
    const INPUT: bool = true;

    fn desc(&self) -> *mut c_char {
        self.0.desc
    }
}

impl From<AudioDevice> for AudioSrc {
    fn from(device: AudioDevice) -> Self {
        Self(device)
    }
}

impl Display for AudioSrc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.0.name.as_str())
    }
}

#[derive(Debug)]
pub(crate) struct AudioDst(AudioDevice);

impl SoundDevice for AudioDst {
    const INPUT: bool = false;

    fn desc(&self) -> *mut c_char {
        self.0.desc
    }
}

impl Display for AudioDst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.0.name.as_str())
    }
}

impl From<AudioDevice> for AudioDst {
    fn from(device: AudioDevice) -> Self {
        Self(device)
    }
}

/// An Audio Device (input or output).
#[derive(Debug)]
pub(crate) struct AudioDevice {
    /// Human readable name for the device.
    name: String,
    /// Device descriptor
    desc: *mut c_char,
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        unsafe {
            free(self.desc.cast());
        }
    }
}

/// Return a list of available audio devices.
pub(crate) fn device_list<D: SoundDevice, F: Fn(D) -> T, T>(abstrakt: F) -> Vec<T> {
    super::ALSA.with(|alsa| {
        if let Some(alsa) = alsa {
            device_list_internal(&alsa, abstrakt)
        } else {
            Vec::new()
        }
    })
}

fn device_list_internal<D: SoundDevice, F: Fn(D) -> T, T>(alsa: &Alsa, abstrakt: F) -> Vec<T> {
    let tpcm = CStr::from_bytes_with_nul(b"pcm\0").unwrap();
    let tname = CStr::from_bytes_with_nul(b"NAME\0").unwrap();
    let tdesc = CStr::from_bytes_with_nul(b"DESC\0").unwrap();
    let tioid = CStr::from_bytes_with_nul(b"IOID\0").unwrap();

    let mut hints = MaybeUninit::uninit();
    let mut devices = Vec::new();
    unsafe {
        if (alsa.snd_device_name_hint)(-1, tpcm.as_ptr(), hints.as_mut_ptr()) < 0 {
            return Vec::new();
        }
        let hints = hints.assume_init();
        let mut n = hints;
        while !(*n).is_null() {
            // Allocate 3 C Strings describing device.
            let name = (alsa.snd_device_name_get_hint)(*n, tname.as_ptr());
            let io = (alsa.snd_device_name_get_hint)(*n, tioid.as_ptr());
            assert_ne!(name, std::ptr::null_mut());

            // Convert description to Rust String
            let desc = match CStr::from_ptr(name).to_str() {
                Ok("null") => "Null".to_string(),
                Ok("default") => "Default".to_string(),
                _ => {
                    let desc = (alsa.snd_device_name_get_hint)(*n, tdesc.as_ptr());
                    assert_ne!(desc, std::ptr::null_mut());
                    let rust = CStr::from_ptr(desc).to_string_lossy().to_string();
                    free(desc.cast());
                    rust.replace("\n", ": ")
                }
            };
            // Check device io direction.
            let is_input = io.is_null() || *(io.cast::<u8>()) == b'I';
            let is_output = io.is_null() || *(io.cast::<u8>()) == b'O';
            if !io.is_null() {
                free(io.cast());
            }
            if (D::INPUT && is_input) || (!D::INPUT && is_output) {
                // Add device to list of devices.
                devices.push(abstrakt(D::from(AudioDevice { name: desc, desc: name })));
            }
            n = n.offset(1);
        }
        (alsa.snd_device_name_free_hint)(hints);
    }
    devices
}
