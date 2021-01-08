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

/// Stream Mode
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SndPcmMode {
    /// Blocking mode
    Block = 0,
    /// Non blocking mode
    Nonblock = 1,
    /// Async notification (deprecated)
    Async = 2,
}

/// PCM stream (direction)
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SndPcmStream {
    /// Playback stream
    Playback = 0,
    /// Capture stream
    Capture,
}

/// PCM access type
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SndPcmAccess {
    /// mmap access with simple interleaved channels
    MmapInterleaved = 0,
    /// mmap access with simple non interleaved channels
    MmapNoninterleaved,
    /// mmap access with complex placement
    MmapComplex,
    /// snd_pcm_readi/snd_pcm_writei access
    RwInterleaved,
    /// snd_pcm_readn/snd_pcm_writen access
    RwNoninterleaved,
}

/// PCM sample format
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SndPcmFormat {
    /// Unknown
    Unknown = -1,
    /// Signed 8 bit
    S8 = 0,
    /// Unsigned 8 bit
    U8,
    /// Signed 16 bit Little Endian (a.k.a `S16` in C API)
    S16Le,
    /// Signed 16 bit Big Endian
    S16Be,
    /// Unsigned 16 bit Little Endian (a.k.a `U16` in C API)
    U16Le,
    /// Unsigned 16 bit Big Endian
    U16Be,
    /// Signed 24 bit Little Endian using low three bytes in 32-bit word
    /// (a.k.a `S24` in C API)
    S24Le,
    /// Signed 24 bit Big Endian using low three bytes in 32-bit word
    S24Be,
    /// Unsigned 24 bit Little Endian using low three bytes in 32-bit word
    /// (a.k.a `U24` in C API)
    U24Le,
    /// Unsigned 24 bit Big Endian using low three bytes in 32-bit word
    U24Be,
    /// Signed 32 bit Little Endian (a.k.a `S32` in C API)
    S32Le,
    /// Signed 32 bit Big Endian
    S32Be,
    /// Unsigned 32 bit Little Endian (a.k.a `U32` in C API)
    U32Le,
    /// Unsigned 32 bit Big Endian
    U32Be,
    /// Float 32 bit Little Endian, Range -1.0 to 1.0
    /// (a.k.a `FLOAT` in C API)
    FloatLe,
    /// Float 32 bit Big Endian, Range -1.0 to 1.0
    FloatBe,
    /// Float 64 bit Little Endian, Range -1.0 to 1.0
    /// (a.k.a `FLOAT64` in C API)
    Float64Le,
    /// Float 64 bit Big Endian, Range -1.0 to 1.0
    Float64Be,
    /// IEC-958 Little Endian (a.k.a `IEC958_SUBFRAME` in C API)
    Iec958SubframeLe,
    /// IEC-958 Big Endian
    Iec958SubframeBe,
    /// Mu-Law
    MuLaw,
    /// A-Law
    ALaw,
    /// Ima-ADPCM
    ImaAdpcm,
    /// MPEG
    Mpeg,
    /// GSM
    Gsm,
    /// Signed 20bit Little/Native Endian in 4bytes format, LSB justified
    /// (a.k.a `S20` in C API)
    S20Le,
    /// Signed 20bit Big Endian in 4bytes format, LSB justified
    S20Be,
    /// Unsigned 20bit Little/Native Endian in 4bytes format, LSB justified
    /// (a.k.a `U20` in C API)
    U20Le,
    /// Unsigned 20bit Big Endian in 4bytes format, LSB justified
    U20Be,
    /// Special
    Special = 31,
    /// Signed 24bit Little Endian in 3bytes format
    S243le = 32,
    /// Signed 24bit Big Endian in 3bytes format
    S243be,
    /// Unsigned 24bit Little Endian in 3bytes format
    U243le,
    /// Unsigned 24bit Big Endian in 3bytes format
    U243be,
    /// Signed 20bit Little Endian in 3bytes format
    S203le,
    /// Signed 20bit Big Endian in 3bytes format
    S203be,
    /// Unsigned 20bit Little Endian in 3bytes format
    U203le,
    /// Unsigned 20bit Big Endian in 3bytes format
    U203be,
    /// Signed 18bit Little Endian in 3bytes format
    S183le,
    /// Signed 18bit Big Endian in 3bytes format
    S183be,
    /// Unsigned 18bit Little Endian in 3bytes format
    U183le,
    /// Unsigned 18bit Big Endian in 3bytes format
    U183be,
    /// Signed 16 bit CPU endian
    G72324,
    /// Unsigned 16 bit CPU endian
    G723241b,
    /// Signed 24 bit CPU endian
    G72340,
    /// Unsigned 24 bit CPU endian
    G723401b,
    /// Signed 32 bit CPU endian
    DsdU8,
    /// Unsigned 32 bit CPU endian
    DsdU16Le,
    /// Float 32 bit CPU endian
    DsdU32Le,
    /// Float 64 bit CPU endian
    DsdU16Be,
    /// IEC-958 CPU Endian
    DsdU32Be,
}

/// PCM state
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SndPcmState {
    /// Open
    Open = 0,
    /// Setup installed
    Setup,
    /// Ready to start
    Prepared,
    /// Running
    Running,
    /// Stopped: underrun (playback) or overrun (capture) detected
    Xrun,
    /// Draining: running (playback) or stopped (capture)
    Draining,
    /// Paused
    Paused,
    /// Hardware is suspended
    Suspended,
    /// Hardware is disconnected (Also known as LAST in the C API)
    Disconnected,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct PollFd {
    pub(super) fd: c_int,
    pub(super) events: std::os::raw::c_short,
    pub(super) revents: std::os::raw::c_short,
}

// Link to libasound
dl_api::linker!(extern "C" Alsa "libasound.so.2" {
    // Device
    fn snd_device_name_hint(
        card: c_int,
        iface: *const c_char,
        hints: *mut *mut *mut c_void,
    ) -> c_int;
    fn snd_device_name_get_hint(hint: *const c_void, id: *const c_char)
        -> *mut c_char;
    fn snd_device_name_free_hint(hints: *mut *mut c_void) -> c_int;

    // PCM
    fn snd_pcm_open(pcmp: *mut *mut c_void,
        name: *const c_char,
        stream: SndPcmStream,
        mode: c_int,
    ) -> c_int;
    fn snd_pcm_close(pcm: *mut c_void) -> c_int;
    fn snd_pcm_drop(pcm: *mut c_void) -> c_int;
    fn snd_pcm_prepare(pcm: *mut c_void) -> c_int;
    fn snd_pcm_start(pcm: *mut c_void) -> c_int;
    fn snd_pcm_resume(pcm: *mut c_void) -> c_int;
    fn snd_pcm_state(pcm: *mut c_void) -> SndPcmState;
    fn snd_pcm_readi(
        pcm: *mut c_void,
        buffer: *mut c_void,
        size: c_ulong,
    ) -> c_long;
    fn snd_pcm_writei(
        pcm: *mut c_void,
        buffer: *const c_void,
        size: c_ulong,
    ) -> c_long;
    fn snd_pcm_avail(pcm: *mut c_void) -> c_long;
    fn snd_pcm_avail_update(pcm: *mut c_void) -> c_long;

    // Poll
    fn snd_pcm_poll_descriptors(pcm: *mut c_void, pfds: *mut PollFd, space: c_uint) -> c_int;
    fn snd_pcm_poll_descriptors_count(pcm: *mut c_void) -> c_int;

    // HW Params
    fn snd_pcm_hw_params(pcm: *mut c_void, params: *mut c_void) -> c_int;
    fn snd_pcm_hw_params_free(params: *mut c_void) -> ();
    fn snd_pcm_hw_params_set_rate_near(pcm: *mut c_void, params: *mut c_void, val: *mut c_uint, dir: *mut c_int) -> c_int;
    fn snd_pcm_hw_params_get_rate_numden(params: *mut c_void, rate_num: *mut c_uint, rate_den: *mut c_uint) -> c_int;
    fn snd_pcm_hw_params_any(pcm: *mut c_void, params: *mut c_void) -> c_int;
    fn snd_pcm_hw_params_test_channels(pcm: *mut c_void, params: *mut c_void, val: c_uint) -> c_int;
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
