// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

#![allow(unsafe_code)]

use std::{convert::TryInto, os::raw::{c_void, c_uint, c_int}, mem::MaybeUninit};

use super::ALSA;
use super::super::{SndPcmAccess, SndPcmFormat};

pub(crate) unsafe fn hw_params_set_period_size_near(
    pcm: *mut c_void,
    params: *mut c_void,
    val: *mut c_uint,
    dir: *mut c_int,
) -> Result<(), i64>
{
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
) -> Result<(), i64>
{
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_buffer_size_near)(
            pcm, params, val
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_format(        pcm: *mut c_void,
        params: *mut c_void,
        access: SndPcmFormat) -> Result<(), i64>
{
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_format)(
            pcm, params, access
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_access(        pcm: *mut c_void,
        params: *mut c_void,
        access: SndPcmAccess) -> Result<(), i64>
{
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_access)(
            pcm, params, access
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_set_rate_near(pcm: *mut c_void, params: *mut c_void, val: *mut c_uint, dir: *mut c_int) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_set_rate_near)(
            pcm, params, val, dir
        );
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

pub(crate) unsafe fn hw_params(pcm: *mut c_void, params: *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params)(
            pcm, params
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_malloc(params: *mut *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_malloc)(
            params,
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

pub(crate) unsafe fn hw_params_any(pcm: *mut c_void, params: *mut c_void) -> Result<(), i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Err(0);
        };
        let ret = (alsa.snd_pcm_hw_params_any)(
            pcm,
            params,
        );
        let _: u64 = ret.try_into().map_err(|_| ret)?;
        Ok(())
    })
}

/// Set the configured channel count.
pub(crate) unsafe fn hw_set_channels(pcm: *mut c_void, params: *mut c_void, hw_params: u8) -> Result<(), i64> {
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

/// Get the configured channel count.
///
/// Marked unsafe because requires that one configuration is chosen.
pub(crate) unsafe fn get_channels_max(hw_params: *mut c_void) -> u8 {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return 0;
        };
        let mut count = MaybeUninit::uninit();
        let ret = (alsa.snd_pcm_hw_params_get_channels_max)(
            hw_params,
            count.as_mut_ptr(),
        );
        let count = count.assume_init();
        if ret != 0 {
            return 0;
        }
        count.try_into().unwrap_or(0)
    })
}

/// Get the configured channel count.
///
/// Marked unsafe because requires that one configuration is chosen.
pub(crate) unsafe fn get_channels(hw_params: *mut c_void) -> u8 {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return 0;
        };
        let mut count = MaybeUninit::uninit();
        let ret = (alsa.snd_pcm_hw_params_get_channels)(
            hw_params,
            count.as_mut_ptr(),
        );
        let count = count.assume_init();
        if ret != 0 {
            return 0;
        }
        count.try_into().unwrap_or(0)
    })
}

/// Get the exact configured sample rate from the speaker/microphone.
///
/// Marked unsafe because requires that one configuration is chosen.
pub(crate) unsafe fn get_rate(hw_params: *mut c_void) -> Option<f64> {
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
        let num = num.assume_init();
        let den = den.assume_init();
        let _err: usize = ret.try_into().ok()?;
        Some(num as f64 / den as f64)
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
) -> Result<usize, i64> {
    ALSA.with(|alsa| {
        let alsa = if let Some(alsa) = alsa {
            alsa
        } else {
            return Ok(0);
        };
        let ret = (alsa.snd_pcm_readi)(
            pcm,
            buffer.cast(),
            length.into(),
        );
        Ok(ret.try_into().map_err(|_| -> i64 { ret.into() })?)
    })
}
