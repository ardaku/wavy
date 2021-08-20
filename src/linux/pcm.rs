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
    mem::MaybeUninit,
    os::raw::{c_char, c_int, c_uint, c_void},
};

use super::super::{
    PollFd, SndPcmAccess, SndPcmFormat, SndPcmMode, SndPcmState, SndPcmStream,
};
use super::ALSA;

pub(crate) unsafe fn hw_params_set_period_size_near(
    pcm: *mut c_void,
    params: *mut c_void,
    val: *mut c_uint,
    dir: *mut c_int,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_period_size_near)(
            pcm, params, val, dir,
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_buffer_size_near(
    pcm: *mut c_void,
    params: *mut c_void,
    val: *mut c_uint,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret =
            (alsa.snd_pcm_hw_params_set_buffer_size_near)(pcm, params, val);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_format(
    pcm: *mut c_void,
    params: *mut c_void,
    access: SndPcmFormat,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_format)(pcm, params, access);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_access(
    pcm: *mut c_void,
    params: *mut c_void,
    access: SndPcmAccess,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_access)(pcm, params, access);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_rate_near(
    pcm: *mut c_void,
    params: *mut c_void,
    val: *mut c_uint,
    dir: *mut c_int,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_rate_near)(pcm, params, val, dir);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_free(params: *mut c_void) {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return;
        };
        (alsa.snd_pcm_hw_params_free)(params);
    })
}

pub(crate) unsafe fn hw_params(
    pcm: *mut c_void,
    params: *mut c_void,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params)(pcm, params);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_malloc() -> Result<*mut c_void, i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let mut hwp = MaybeUninit::uninit();
        let ret = (alsa.snd_pcm_hw_params_malloc)(hwp.as_mut_ptr());
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        let hwp = hwp.assume_init();
        Ok(hwp)
    })
}

pub(crate) unsafe fn hw_params_any(
    pcm: *mut c_void,
    params: *mut c_void,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_any)(pcm, params);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

/// Set the configured channel count.
pub(crate) unsafe fn hw_set_channels(
    pcm: *mut c_void,
    params: *mut c_void,
    hw_params: u8,
) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_channels)(
            pcm,
            params,
            hw_params.into(),
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

/// Get the exact configured sample rate from the speaker/microphone.
///
/// Marked unsafe because requires that one configuration is chosen.
pub(crate) unsafe fn hw_get_rate(hw_params: *mut c_void) -> Option<f64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return None;
        };
        let mut num = MaybeUninit::uninit();
        let mut den = MaybeUninit::uninit();
        let ret = (alsa.snd_pcm_hw_params_get_rate_numden)(
            hw_params,
            num.as_mut_ptr(),
            den.as_mut_ptr(),
        );
        let _err: usize = ret.try_into().ok()?;
        let num = num.assume_init();
        let den = den.assume_init();
        Some(num as f64 / den as f64)
    })
}

pub(crate) unsafe fn poll_descriptors(
    pcm: *mut c_void,
) -> Result<Vec<PollFd>, i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let size: usize = (alsa.snd_pcm_poll_descriptors_count)(pcm)
            .try_into()
            .unwrap();
        let mut poll = Vec::with_capacity(size);
        let ret = (alsa.snd_pcm_poll_descriptors)(
            pcm,
            poll.as_mut_ptr(),
            size.try_into().unwrap(),
        );
        poll.set_len(size);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(poll)
    })
}

pub(crate) unsafe fn close(pcm: *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_close)(pcm);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn drop(pcm: *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_drop)(pcm);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn resume(pcm: *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_resume)(pcm);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn prepare(pcm: *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_prepare)(pcm);
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn state(pcm: *mut c_void) -> SndPcmState {
    ALSA.with(|alsa| {
        let alsa = alsa.as_ref().unwrap();
        (alsa.snd_pcm_state)(pcm)
    })
}

/// Read microphone input into an audio frame buffer.
///
/// Marked unsafe because pcm must be configured to handle interleaved frames
/// the size of `F` to prevent undefined behavior.
pub(crate) unsafe fn readi<T>(
    pcm: *mut c_void,
    buffer: *mut T,
    length: u16,
) -> Result<usize, isize> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Ok(0);
        };
        let ret = (alsa.snd_pcm_readi)(pcm, buffer.cast(), length.into());
        Ok(ret.try_into().map_err(|_| -> isize { ret as isize })?)
    })
}

/// Write speaker output from an audio frame buffer.
///
/// Marked unsafe because pcm must be configured to handle interleaved frames
/// the size of `F` to prevent undefined behavior.
pub(crate) unsafe fn writei<T>(
    pcm: *mut c_void,
    buffer: *const T,
    length: usize,
) -> Result<usize, isize> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Ok(0);
        };
        let ret = (alsa.snd_pcm_writei)(pcm, buffer.cast(), length as _);
        Ok(ret.try_into().map_err(|_| -> isize { ret as isize })?)
    })
}
