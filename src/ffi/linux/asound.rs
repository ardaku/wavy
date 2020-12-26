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

use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

use super::{SndPcmAccess, SndPcmFormat};

// Link to libasound
dl_api::linker!(extern "C" Alsa "libasound.so.2" {
    fn snd_device_name_hint(
        card: c_int,
        iface: *const c_char,
        hints: *mut *mut *mut c_void,
    ) -> c_int;
    fn snd_device_name_get_hint(hint: *const c_void, id: *const c_char)
        -> *mut c_char;
    fn snd_device_name_free_hint(hints: *mut *mut c_void) -> c_int;
    fn snd_pcm_readi(
        pcm: *mut c_void,
        buffer: *mut c_void,
        size: c_ulong,
    ) -> c_long;

    // HW Params
    fn snd_pcm_hw_params(pcm: *mut c_void, params: *mut c_void) -> c_int;
    fn snd_pcm_hw_params_free(params: *mut c_void) -> ();
    fn snd_pcm_hw_params_set_rate_near(pcm: *mut c_void, params: *mut c_void, val: *mut c_uint, dir: *mut c_int) -> c_int;
    fn snd_pcm_hw_params_get_rate_numden(params: *mut c_void, rate_num: *mut c_uint, rate_den: *mut c_uint) -> c_int;
    fn snd_pcm_hw_params_any(pcm: *mut c_void, params: *mut c_void) -> c_int;
    fn snd_pcm_hw_params_set_channels(pcm: *mut c_void, params: *mut c_void, val: c_uint) -> c_int;
    fn snd_pcm_hw_params_malloc(ptr: *mut *mut c_void) -> c_int;
    fn snd_pcm_hw_params_set_access(
        pcm: *mut c_void,
        params: *mut c_void,
        access: SndPcmAccess,
    ) -> c_int;
    fn snd_pcm_hw_params_set_format(
        pcm: *mut c_void,
        params: *mut c_void,
        access: SndPcmFormat,
    ) -> c_int;
    fn snd_pcm_hw_params_set_buffer_size_near(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
    ) -> c_int;
    fn snd_pcm_hw_params_set_period_size_near(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int;
});

//
extern "C" {
    pub(super) fn free(ptr: *mut c_void);
}

thread_local! {
    static ALSA: Option<Alsa> = Alsa::new().ok();
}

#[path = "device_list.rs"]
pub(super) mod device_list;
#[path = "pcm.rs"]
pub(super) mod pcm;
