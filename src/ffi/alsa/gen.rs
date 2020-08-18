// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(clippy::unused_unit)]
#![allow(clippy::let_unit_value)]
#![allow(unsafe_code)]
#![allow(trivial_numeric_casts, trivial_casts)]

use std::os::raw::{c_uint, c_void, c_int};

const LM_ID_NEWLM: std::os::raw::c_long = -1;
const RTLD_NOW: c_int = 0x00002;

extern "C" {
    fn dlmopen(
        lmid: std::os::raw::c_long,
        filename: *const std::os::raw::c_char,
        flags: c_int,
    ) -> *mut c_void;
    fn dlsym(
        handle: *mut c_void,
        symbol: *const std::os::raw::c_char,
    ) -> *mut c_void;
}

unsafe fn new(name: &[u8]) -> Option<std::ptr::NonNull<c_void>> {
    std::ptr::NonNull::new(dlmopen(LM_ID_NEWLM, name.as_ptr().cast(), RTLD_NOW))
}

unsafe fn sym(
    dll: std::ptr::NonNull<c_void>,
    name: &[u8],
) -> Option<std::ptr::NonNull<c_void>> {
    std::ptr::NonNull::new(dlsym(dll.as_ptr(), name.as_ptr().cast()))
}

static mut THREAD_ID: std::mem::MaybeUninit<std::thread::ThreadId> =
    std::mem::MaybeUninit::uninit();
static mut DLL: std::mem::MaybeUninit<std::ptr::NonNull<c_void>> =
    std::mem::MaybeUninit::uninit();
static mut START_FFI: std::sync::Once = std::sync::Once::new();
static mut SUCCESS: bool = false;

unsafe fn check_thread() -> Option<std::ptr::NonNull<c_void>> {
    START_FFI.call_once(|| {
        THREAD_ID = std::mem::MaybeUninit::new(std::thread::current().id());
        if let Some(dll) = new(DL_API_SHARED_OBJECT_NAME) {
            DLL = std::mem::MaybeUninit::new(dll);
            SUCCESS = true;
        }
    });

    assert_eq!(THREAD_ID.assume_init(), std::thread::current().id());

    if SUCCESS {
        Some(DLL.assume_init())
    } else {
        None
    }
}

const DL_API_SHARED_OBJECT_NAME: &[u8] = b"libasound.so.2\0";

/// Stream Mode
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum SndPcmMode {
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
pub(super) enum SndPcmStream {
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
pub(super) enum SndPcmAccess {
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
pub(super) enum SndPcmFormat {
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
pub(super) enum SndPcmState {
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

/// PCM handle
pub(super) struct SndPcm(*mut c_void);

/// PCM hardware configuration space container
///
/// snd_pcm_hw_params_t is an opaque structure which contains a set of
/// possible PCM hardware configurations. For example, a given instance might
/// include a range of buffer sizes, a range of period sizes, and a set of
/// several sample formats. Some subset of all possible combinations these
/// sets may be valid, but not necessarily any combination will be valid.
///
/// When a parameter is set or restricted using a snd_pcm_hw_params_set*
/// function, all of the other ranges will be updated to exclude as many
/// impossible configurations as possible. Attempting to set a parameter
/// outside of its acceptable range will result in the function failing and
/// an error code being returned.
pub(super) struct SndPcmHwParams(*mut c_void);

#[repr(C)]
#[derive(Copy, Clone)]
pub(super) struct PollFd {
    pub(super) fd: c_int,
    pub(super) events: std::os::raw::c_short,
    pub(super) revents: std::os::raw::c_short,
}

static mut FN_SND_PCM_OPEN: std::mem::MaybeUninit<
    extern "C" fn(
        pcmp: *mut *mut c_void,
        name: *const std::os::raw::c_char,
        stream: SndPcmStream,
        mode: SndPcmMode,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_MALLOC: std::mem::MaybeUninit<
    extern "C" fn(ptr: *mut *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_ANY: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void, params: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_RATE_RESAMPLE: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        val: c_uint,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_ACCESS: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        access: SndPcmAccess,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_FORMAT: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        format: SndPcmFormat,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_CHANNELS: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        val: c_uint,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_RATE_NEAR: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_PERIOD_SIZE_NEAR: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_BUFFER_SIZE_NEAR: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void, params: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_GET_PERIOD_SIZE: std::mem::MaybeUninit<
    extern "C" fn(
        params: *const c_void,
        frames: *mut std::os::raw::c_ulong,
        dir: *mut c_int,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_FREE: std::mem::MaybeUninit<
    extern "C" fn(obj: *mut c_void) -> (),
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_WRITEI: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        buffer: *const u32,
        size: std::os::raw::c_ulong,
    ) -> std::os::raw::c_long,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_READI: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        buffer: *mut u32,
        size: std::os::raw::c_ulong,
    ) -> std::os::raw::c_long,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_CLOSE: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_POLL_DESCRIPTORS_COUNT: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_POLL_DESCRIPTORS: std::mem::MaybeUninit<
    extern "C" fn(
        pcm: *mut c_void,
        pfds: *mut PollFd,
        space: c_uint,
    ) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_STATE: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void) -> SndPcmState,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_DROP: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_PREPARE: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_RESUME: std::mem::MaybeUninit<
    extern "C" fn(pcm: *mut c_void) -> c_int,
> = std::mem::MaybeUninit::uninit();

static mut ALSA_DEVICE_INIT: Option<AlsaDevice> = None;

/// A module contains functions.
#[derive(Clone)]
pub(super) struct AlsaDevice(std::marker::PhantomData<*mut u8>);

impl AlsaDevice {
    /// Get a handle to this module.  Loads module functions on first call.
    pub(super) fn new() -> Option<Self> {
        unsafe {
            let dll = check_thread()?;
            if let Some(ref module) = ALSA_DEVICE_INIT {
                return Some(module.clone());
            }
            FN_SND_PCM_OPEN = std::mem::MaybeUninit::new(std::mem::transmute(
                sym(dll, b"snd_pcm_open\0")?.as_ptr(),
            ));
            FN_SND_PCM_HW_PARAMS_MALLOC =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_malloc\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_ANY =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_any\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_RATE_RESAMPLE =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_rate_resample\0")?
                        .as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_ACCESS =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_access\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_FORMAT =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_format\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_CHANNELS =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_channels\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_RATE_NEAR =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_rate_near\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_PERIOD_SIZE_NEAR =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_period_size_near\0")?
                        .as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_SET_BUFFER_SIZE_NEAR =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_set_buffer_size_near\0")?
                        .as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS = std::mem::MaybeUninit::new(
                std::mem::transmute(sym(dll, b"snd_pcm_hw_params\0")?.as_ptr()),
            );
            FN_SND_PCM_HW_PARAMS_GET_PERIOD_SIZE =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_get_period_size\0")?.as_ptr(),
                ));
            FN_SND_PCM_HW_PARAMS_FREE =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_hw_params_free\0")?.as_ptr(),
                ));
            FN_SND_PCM_CLOSE = std::mem::MaybeUninit::new(std::mem::transmute(
                sym(dll, b"snd_pcm_close\0")?.as_ptr(),
            ));
            FN_SND_PCM_POLL_DESCRIPTORS_COUNT =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_poll_descriptors_count\0")?.as_ptr(),
                ));
            FN_SND_PCM_POLL_DESCRIPTORS =
                std::mem::MaybeUninit::new(std::mem::transmute(
                    sym(dll, b"snd_pcm_poll_descriptors\0")?.as_ptr(),
                ));
            FN_SND_PCM_STATE = std::mem::MaybeUninit::new(std::mem::transmute(
                sym(dll, b"snd_pcm_state\0")?.as_ptr(),
            ));
            FN_SND_PCM_DROP = std::mem::MaybeUninit::new(std::mem::transmute(
                sym(dll, b"snd_pcm_drop\0")?.as_ptr(),
            ));
            FN_SND_PCM_PREPARE = std::mem::MaybeUninit::new(
                std::mem::transmute(sym(dll, b"snd_pcm_prepare\0")?.as_ptr()),
            );
            FN_SND_PCM_RESUME = std::mem::MaybeUninit::new(
                std::mem::transmute(sym(dll, b"snd_pcm_resume\0")?.as_ptr()),
            );
            ALSA_DEVICE_INIT = Some(Self(std::marker::PhantomData));
            Some(Self(std::marker::PhantomData))
        }
    }
    /// Opens a PCM.
    /// - `pcmp`: Returned PCM handle
    /// - `name`: ASCII identifier of the PCM handle
    /// - `stream`: Wanted stream
    /// - `mode`: Open mode
    /// Return 0 on success otherwise a negative error code
    pub(super) fn snd_pcm_open(
        &self,
        name: &std::ffi::CStr,
        stream: SndPcmStream,
        mode: SndPcmMode,
    ) -> Result<SndPcm, i32> {
        unsafe {
            let mut pcmp = std::mem::MaybeUninit::uninit();
            let __ret = ((FN_SND_PCM_OPEN).assume_init())(
                pcmp.as_mut_ptr(),
                name.as_ptr(),
                stream as _,
                mode as _,
            );
            let pcmp = pcmp.assume_init();
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(SndPcm(pcmp) as _)
        }
    }
    /// Allocate an invalid snd_pcm_hw_params_t using standard malloc
    pub(super) fn snd_pcm_hw_params_malloc(
        &self,
    ) -> Result<SndPcmHwParams, i32> {
        unsafe {
            let mut ptr = std::mem::MaybeUninit::uninit();
            let __ret =
                ((FN_SND_PCM_HW_PARAMS_MALLOC).assume_init())(ptr.as_mut_ptr());
            let ptr = ptr.assume_init();
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(SndPcmHwParams(ptr) as _)
        }
    }
    /// Fill params with a full configuration space for a PCM.
    pub(super) fn snd_pcm_hw_params_any(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
    ) -> Result<(), i32> {
        unsafe {
            let __ret =
                ((FN_SND_PCM_HW_PARAMS_ANY).assume_init())(pcm.0, params.0);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only real hardware rates.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space
    /// - `val`: 0 = disable, 1 = enable (default) rate resampling
    pub(super) fn snd_pcm_hw_params_set_rate_resample(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: u32,
    ) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_RATE_RESAMPLE)
                .assume_init())(
                pcm.0, params.0, val as _
            );
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only one access type.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space
    /// - `access`: access type
    pub(super) fn snd_pcm_hw_params_set_access(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        access: SndPcmAccess,
    ) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_ACCESS).assume_init())(
                pcm.0,
                params.0,
                access as _,
            );
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only one format.
    pub(super) fn snd_pcm_hw_params_set_format(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        format: SndPcmFormat,
    ) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_FORMAT).assume_init())(
                pcm.0,
                params.0,
                format as _,
            );
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only one channels count.
    pub(super) fn snd_pcm_hw_params_set_channels(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        value: u32,
    ) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_CHANNELS).assume_init())(
                pcm.0, params.0, value as _,
            );
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to have rate nearest to a target.
    ///  - `pcm`: PCM handle
    ///  - `params`: Configuration space
    ///  - `val`: approximate target rate / returned approximate set rate
    ///  - `dir`: Sub unit direction
    pub(super) fn snd_pcm_hw_params_set_rate_near(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: &mut u32,
        dir: Option<&mut i32>,
    ) -> Result<(), i32> {
        unsafe {
            let mut __val: _ = *val as _;
            let mut __dir: _ = if let Some(_temp) = dir.iter().next() {
                Some(**_temp as _)
            } else {
                None
            };
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_RATE_NEAR).assume_init())(
                pcm.0,
                params.0,
                &mut __val,
                if let Some(ref mut _temp) = __dir {
                    _temp
                } else {
                    std::ptr::null_mut()
                },
            );
            *val = __val as _;
            if let Some(_temp) = dir {
                *_temp = __dir.unwrap() as _;
            }
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to have period size nearest to a target.
    ///  - `pcm`: PCM handle
    ///  - `params`: Configuration space
    ///  - `val`: approximate target period size in frames / returned chosen
    ///    approximate target period size
    ///  - `dir`: Sub unit direction
    pub(super) fn snd_pcm_hw_params_set_period_size_near(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: &mut u32,
        dir: Option<&mut i32>,
    ) -> Result<(), i32> {
        unsafe {
            let mut __val: _ = *val as _;
            let mut __dir: _ = if let Some(_temp) = dir.iter().next() {
                Some(**_temp as _)
            } else {
                None
            };
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_PERIOD_SIZE_NEAR)
                .assume_init())(
                pcm.0,
                params.0,
                &mut __val,
                if let Some(ref mut _temp) = __dir {
                    _temp
                } else {
                    std::ptr::null_mut()
                },
            );
            *val = __val as _;
            if let Some(_temp) = dir {
                *_temp = __dir.unwrap() as _;
            }
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Restrict a configuration space to have buffer size nearest to a target.
    ///  - `pcm`: PCM handle
    ///  - `params`: Configuration space
    ///  - `val`: Approximate target buffer size in frames / returned chosen
    ///    approximate target buffer size in frames
    pub(super) fn snd_pcm_hw_params_set_buffer_size_near(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: &mut u32,
    ) -> Result<(), i32> {
        unsafe {
            let mut __val: _ = *val as _;
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_BUFFER_SIZE_NEAR)
                .assume_init())(
                pcm.0, params.0, &mut __val
            );
            *val = __val as _;
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Install one PCM hardware configuration chosen from a configuration space
    /// and snd_pcm_prepare it.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space definition container
    pub(super) fn snd_pcm_hw_params(
        &self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
    ) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS).assume_init())(pcm.0, params.0);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Frees a previously allocated snd_pcm_hw_params_t
    pub(super) fn snd_pcm_hw_params_free(
        &self,
        obj: &mut SndPcmHwParams,
    ) -> () {
        unsafe {
            if obj.0.is_null() {
                panic!("Object free'd twice!")
            }
            let __ret = ((FN_SND_PCM_HW_PARAMS_FREE).assume_init())(obj.0);
            obj.0 = std::ptr::null_mut();
            ()
        }
    }
    /// close PCM handle
    pub(super) fn snd_pcm_close(&self, pcm: &mut SndPcm) -> Result<(), i32> {
        unsafe {
            if pcm.0.is_null() {
                panic!("Object free'd twice!")
            }
            let __ret = ((FN_SND_PCM_CLOSE).assume_init())(pcm.0);
            pcm.0 = std::ptr::null_mut();
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Get count of poll descriptors for PCM handle
    pub(super) fn snd_pcm_poll_descriptors_count(
        &self,
        pcm: &SndPcm,
    ) -> Result<i32, i32> {
        unsafe {
            let __ret =
                ((FN_SND_PCM_POLL_DESCRIPTORS_COUNT).assume_init())(pcm.0);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(__ret as _)
        }
    }
    /// Get poll descriptors
    ///  - `pcm`: PCM handle
    ///  - `pfds`: Array of poll descriptors
    ///  - `space`: space in the poll descriptor array
    ///  -Returns count of filled descriptors
    pub(super) fn snd_pcm_poll_descriptors(
        &self,
        pcm: &SndPcm,
        pfds: &mut Vec<PollFd>,
    ) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_POLL_DESCRIPTORS).assume_init())(
                pcm.0,
                pfds.as_mut_ptr(),
                pfds.capacity() as _,
            );
            pfds.set_len(__ret as _);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Return PCM state.
    ///  - `pcm`: PCM handle
    pub(super) fn snd_pcm_state(&self, pcm: &SndPcm) -> SndPcmState {
        unsafe {
            let __ret = ((FN_SND_PCM_STATE).assume_init())(pcm.0);
            __ret as _
        }
    }
    /// Stop a PCM dropping pending frames.  This function stops the PCM
    /// immediately. The pending samples on the buffer are ignored
    ///  - `pcm`: PCM handle
    pub(super) fn snd_pcm_drop(&self, pcm: &SndPcm) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_DROP).assume_init())(pcm.0);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Prepare PCM for use.
    ///  - `pcm`: PCM handle
    pub(super) fn snd_pcm_prepare(&self, pcm: &SndPcm) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_PREPARE).assume_init())(pcm.0);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
    /// Resume from suspend, no samples are lost.
    ///  - `pcm`: PCM handle
    pub(super) fn snd_pcm_resume(&self, pcm: &SndPcm) -> Result<(), i32> {
        unsafe {
            let __ret = ((FN_SND_PCM_RESUME).assume_init())(pcm.0);
            if __ret < 0 {
                return Err(__ret as _);
            };
            Ok(())
        }
    }
}

static mut ALSA_PLAYER_INIT: Option<AlsaPlayer> = None;

/// A module contains functions.
#[derive(Clone)]
pub(super) struct AlsaPlayer(std::marker::PhantomData<*mut u8>);

impl AlsaPlayer {
    /// Get a handle to this module.  Loads module functions on first call.
    pub(super) fn new() -> Option<Self> {
        unsafe {
            let dll = check_thread()?;
            if let Some(ref module) = ALSA_PLAYER_INIT {
                return Some(module.clone());
            }
            FN_SND_PCM_WRITEI = std::mem::MaybeUninit::new(
                std::mem::transmute(sym(dll, b"snd_pcm_writei\0")?.as_ptr()),
            );
            ALSA_PLAYER_INIT = Some(Self(std::marker::PhantomData));
            Some(Self(std::marker::PhantomData))
        }
    }
    /// Write interleaved frames to a PCM.
    /// - `pcm`: PCM handle
    /// - `buffer`: frames containing buffer
    /// - `size`: frames to be written
    /// If the blocking behaviour is selected and it is running, then routine
    /// waits until all requested frames are played or put to the playback ring
    /// buffer. The returned number of frames can be less only if a signal or
    /// underrun occurred.
    ///
    /// If the non-blocking behaviour is selected, then routine doesn't wait at
    /// all.
    pub(super) unsafe fn snd_pcm_writei(
        &self,
        pcm: &SndPcm,
        frames: *const c_void,
        f_size: usize,
    ) -> Result<isize, isize> {
        let __ret = ((FN_SND_PCM_WRITEI).assume_init())(
            pcm.0,
            frames as _,
            f_size as _,
        );
        if __ret < 0 {
            Err(__ret as _)
        } else {
            Ok(__ret as _)
        }
    }
}

static mut ALSA_RECORDER_INIT: Option<AlsaRecorder> = None;

/// A module contains functions.
#[derive(Clone)]
pub(super) struct AlsaRecorder(std::marker::PhantomData<*mut u8>);

impl AlsaRecorder {
    /// Get a handle to this module.  Loads module functions on first call.
    pub(super) fn new() -> Option<Self> {
        unsafe {
            let dll = check_thread()?;
            if let Some(ref module) = ALSA_RECORDER_INIT {
                return Some(module.clone());
            }
            FN_SND_PCM_READI = std::mem::MaybeUninit::new(std::mem::transmute(
                sym(dll, b"snd_pcm_readi\0")?.as_ptr(),
            ));
            ALSA_RECORDER_INIT = Some(Self(std::marker::PhantomData));
            Some(Self(std::marker::PhantomData))
        }
    }
    /// Read interleaved frames from a PCM.
    /// - `pcm`: PCM handle
    /// - `buffer`: frames containing buffer
    /// - `size`: frames to be read
    /// If the blocking behaviour was selected and it is running, then routine
    /// waits until all requested frames are filled. The returned number of
    /// frames can be less only if a signal or underrun occurred.
    ///
    /// If the non-blocking behaviour is selected, then routine doesn't wait at
    /// all.
    pub(super) fn snd_pcm_readi(
        &self,
        pcm: &SndPcm,
        buffer: &mut Vec<[u8; 2]>,
    ) -> Result<(), isize> {
        let len = buffer.len();
        unsafe {
            let __ret = ((FN_SND_PCM_READI).assume_init())(
                pcm.0,
                buffer[len..].as_mut_ptr() as _,
                (buffer.capacity() - len) as _,
            );
            if __ret < 0 {
                return Err(__ret as _);
            };
            buffer.set_len(len + __ret as usize);
            Ok(())
        }
    }
}
