#![allow(non_camel_case_types)]

use libc;
use libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void, size_t};

pub(crate) enum snd_pcm_status_t {}
pub(crate) enum snd_pcm_t {}
pub(crate) enum snd_pcm_hw_params_t {}

pub(crate) type snd_pcm_sframes_t = c_long;
pub(crate) type snd_pcm_format_t = c_int;
pub(crate) type snd_pcm_access_t = c_uint;
pub(crate) type snd_pcm_stream_t = c_uint;

pub(crate) struct Context {
    pub(crate) snd_pcm_status_get_avail:
        unsafe extern "C" fn(obj: *const snd_pcm_status_t) -> c_ulong,
    pub(crate) snd_pcm_status_sizeof: unsafe extern "C" fn() -> size_t,
    pub(crate) snd_pcm_hw_params_malloc:
        unsafe extern "C" fn(ptr: *mut *mut snd_pcm_hw_params_t) -> c_int,
    pub(crate) snd_pcm_hw_params_free: unsafe extern "C" fn(obj: *mut snd_pcm_hw_params_t),
    pub(crate) snd_pcm_hw_params_get_buffer_size:
        unsafe extern "C" fn(params: *const snd_pcm_hw_params_t, val: *mut c_ulong) -> c_int,
    pub(crate) snd_pcm_hw_params_get_period_size: unsafe extern "C" fn(
        params: *const snd_pcm_hw_params_t,
        frames: *mut c_ulong,
        dir: *mut c_int,
    ) -> c_int,
    pub(crate) snd_pcm_hw_params_set_access: unsafe extern "C" fn(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        _access: snd_pcm_access_t,
    ) -> c_int,
    pub(crate) snd_pcm_hw_params_set_format: unsafe extern "C" fn(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: snd_pcm_format_t,
    ) -> c_int,
    pub(crate) snd_pcm_hw_params_get_rate: unsafe extern "C" fn(
        params: *const snd_pcm_hw_params_t,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int,
    pub(crate) snd_pcm_hw_params_set_rate: unsafe extern "C" fn(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: c_uint,
        dir: c_int,
    ) -> c_int,
    pub(crate) snd_pcm_hw_params_get_channels:
        unsafe extern "C" fn(params: *const snd_pcm_hw_params_t, val: *mut c_uint) -> c_int,
    pub(crate) snd_pcm_hw_params_set_channels: unsafe extern "C" fn(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: c_uint,
    ) -> c_int,
    pub(crate) snd_pcm_hw_params_any:
        unsafe extern "C" fn(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> c_int,
    pub(crate) snd_pcm_writei: unsafe extern "C" fn(
        pcm: *mut snd_pcm_t,
        buffer: *const c_void,
        size: c_ulong,
    ) -> snd_pcm_sframes_t,
    pub(crate) snd_pcm_readi: unsafe extern "C" fn(
        pcm: *mut snd_pcm_t,
        buffer: *mut c_void,
        size: c_ulong,
    ) -> snd_pcm_sframes_t,
    pub(crate) snd_pcm_close: unsafe extern "C" fn(pcm: *mut snd_pcm_t) -> c_int,
    pub(crate) snd_pcm_hw_params_current:
        unsafe extern "C" fn(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> c_int,
    pub(crate) snd_pcm_hw_params:
        unsafe extern "C" fn(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> c_int,
    pub(crate) snd_pcm_status:
        unsafe extern "C" fn(pcm: *mut snd_pcm_t, status: *mut snd_pcm_status_t) -> c_int,
    pub(crate) snd_pcm_open: unsafe extern "C" fn(
        pcm: *mut *mut snd_pcm_t,
        name: *const c_char,
        stream: snd_pcm_stream_t,
        mode: c_int,
    ) -> c_int,
    pub(crate) snd_pcm_recover:
        unsafe extern "C" fn(pcm: *mut snd_pcm_t, err: c_int, silent: c_int) -> c_int,
    pub(crate) snd_pcm_prepare: unsafe extern "C" fn(pcm: *mut snd_pcm_t) -> c_int,
    pub(crate) snd_pcm_start: unsafe extern "C" fn(pcm: *mut snd_pcm_t) -> c_int,
}

fn dlsym<T>(lib: *mut c_void, name: &[u8]) -> T {
    unsafe {
        let fn_ptr = libc::dlsym(lib, &name[0] as *const _ as *const _);

        ::std::mem::transmute_copy::<*mut c_void, T>(&fn_ptr)
    }
}

impl Context {
    pub(crate) fn new() -> Self {
        let lib = b"libasound.so.2\0";
        let lib = unsafe {
            libc::dlopen(
                &lib[0] as *const _ as *const _,
                libc::RTLD_NOW | libc::RTLD_GLOBAL,
            )
        };

        Context {
            snd_pcm_status_get_avail: dlsym(lib, b"snd_pcm_status_get_avail\0"),
            snd_pcm_status_sizeof: dlsym(lib, b"snd_pcm_status_sizeof\0"),
            snd_pcm_hw_params_malloc: dlsym(lib, b"snd_pcm_hw_params_malloc\0"),
            snd_pcm_hw_params_free: dlsym(lib, b"snd_pcm_hw_params_free\0"),
            snd_pcm_hw_params_get_buffer_size: dlsym(lib, b"snd_pcm_hw_params_get_buffer_size\0"),
            snd_pcm_hw_params_get_period_size: dlsym(lib, b"snd_pcm_hw_params_get_period_size\0"),
            snd_pcm_hw_params_set_access: dlsym(lib, b"snd_pcm_hw_params_set_access\0"),
            snd_pcm_hw_params_set_format: dlsym(lib, b"snd_pcm_hw_params_set_format\0"),
            snd_pcm_hw_params_get_rate: dlsym(lib, b"snd_pcm_hw_params_get_rate\0"),
            snd_pcm_hw_params_set_rate: dlsym(lib, b"snd_pcm_hw_params_set_rate\0"),
            snd_pcm_hw_params_get_channels: dlsym(lib, b"snd_pcm_hw_params_get_channels\0"),
            snd_pcm_hw_params_set_channels: dlsym(lib, b"snd_pcm_hw_params_set_channels\0"),
            snd_pcm_hw_params_any: dlsym(lib, b"snd_pcm_hw_params_any\0"),
            snd_pcm_writei: dlsym(lib, b"snd_pcm_writei\0"),
            snd_pcm_readi: dlsym(lib, b"snd_pcm_readi\0"),
            snd_pcm_close: dlsym(lib, b"snd_pcm_close\0"),
            snd_pcm_hw_params_current: dlsym(lib, b"snd_pcm_hw_params_current\0"),
            snd_pcm_hw_params: dlsym(lib, b"snd_pcm_hw_params\0"),
            snd_pcm_status: dlsym(lib, b"snd_pcm_status\0"),
            snd_pcm_open: dlsym(lib, b"snd_pcm_open\0"),
            snd_pcm_recover: dlsym(lib, b"snd_pcm_recover\0"),
            snd_pcm_prepare: dlsym(lib, b"snd_pcm_prepare\0"),
            snd_pcm_start: dlsym(lib, b"snd_pcm_start\0"),
        }
    }
}
