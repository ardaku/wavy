//! Generated by DL API

#![allow(clippy::unused_unit)]
#![allow(clippy::let_unit_value)]
#![allow(unused)]
#![allow(unsafe_code)]
// #![rustfmt::skip] // non-builtin inner attributes are unstable

const LM_ID_NEWLM: std::os::raw::c_long = -1;
const RTLD_NOW: std::os::raw::c_int = 0x00002;

extern {
    fn dlmopen(
        lmid: std::os::raw::c_long,
        filename: *const std::os::raw::c_char,
        flags: std::os::raw::c_int
    ) -> *mut std::ffi::c_void;
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const std::os::raw::c_char)
        -> *mut std::ffi::c_void;
}

unsafe fn new(name: &[u8]) -> Option<std::ptr::NonNull<std::ffi::c_void>> {
    std::ptr::NonNull::new(dlmopen(LM_ID_NEWLM, name.as_ptr().cast(), RTLD_NOW))
}

unsafe fn sym(dll: std::ptr::NonNull<std::ffi::c_void>, name: &[u8])
    -> Option<std::ptr::NonNull<std::ffi::c_void>>
{
    std::ptr::NonNull::new(dlsym(dll.as_ptr(), name.as_ptr().cast()))
}

static mut THREAD_ID: std::mem::MaybeUninit<std::thread::ThreadId>
    = std::mem::MaybeUninit::uninit();
static mut DLL: std::mem::MaybeUninit<std::ptr::NonNull<std::ffi::c_void>>
    = std::mem::MaybeUninit::uninit();
static mut START_FFI: std::sync::Once = std::sync::Once::new();
static mut SUCCESS: bool = false;

unsafe fn check_thread() -> Option<std::ptr::NonNull<std::ffi::c_void>> {
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
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SndPcmMode {
    /// Blocking mode
    Block = 0,
    /// Non blocking mode
    Nonblock = 1,
    /// Async notification (deprecated)
    Async = 2,
}

/// PCM stream (direction)
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SndPcmStream {
    /// Playback stream
    Playback = 0,
    /// Capture stream
    Capture,
}

/// PCM access type
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SndPcmAccess {
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
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SndPcmFormat {
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

/// PCM handle
pub struct SndPcm(*mut std::os::raw::c_void);

impl SndPcm {
    /// Create address struct from raw pointer.
    pub unsafe fn from_raw(raw: *mut std::os::raw::c_void) -> Self {
        Self(raw)
    }
}

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
pub struct SndPcmHwParams(*mut std::os::raw::c_void);

impl SndPcmHwParams {
    /// Create address struct from raw pointer.
    pub unsafe fn from_raw(raw: *mut std::os::raw::c_void) -> Self {
        Self(raw)
    }
}

/// PCM status container
pub struct SndPcmStatus(*mut std::os::raw::c_void);

impl SndPcmStatus {
    /// Create address struct from raw pointer.
    pub unsafe fn from_raw(raw: *mut std::os::raw::c_void) -> Self {
        Self(raw)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PollFd {
    pub fd: std::os::raw::c_int,
    pub events: std::os::raw::c_short,
    pub revents: std::os::raw::c_short,
}

static mut FN_SND_PCM_OPEN:
    std::mem::MaybeUninit<extern fn(
        pcmp: *mut *mut std::os::raw::c_void,
        name: *const std::os::raw::c_char,
        stream: SndPcmStream,
        mode: SndPcmMode,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_MALLOC:
    std::mem::MaybeUninit<extern fn(
        ptr: *mut *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_ANY:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_RATE_RESAMPLE:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
        val: std::os::raw::c_uint,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_ACCESS:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
        access: SndPcmAccess,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_FORMAT:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
        format: SndPcmFormat,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_CHANNELS:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
        val: std::os::raw::c_uint,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_SET_RATE_NEAR:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
        val: *mut std::os::raw::c_uint,
        dir: *mut std::os::raw::c_int,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        params: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_GET_BUFFER_SIZE:
    std::mem::MaybeUninit<extern fn(
        params: *const std::os::raw::c_void,
        val: *mut std::os::raw::c_ulong,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_GET_PERIOD_SIZE:
    std::mem::MaybeUninit<extern fn(
        params: *const std::os::raw::c_void,
        frames: *mut std::os::raw::c_ulong,
        dir: *mut std::os::raw::c_int,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_HW_PARAMS_FREE:
    std::mem::MaybeUninit<extern fn(
        obj: *mut std::os::raw::c_void,
    ) -> ()> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_PREPARE:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_WRITEI:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        buffer: *const u32,
        size: std::os::raw::c_ulong,
    ) -> std::os::raw::c_long> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_READI:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        buffer: *mut u32,
        size: std::os::raw::c_ulong,
    ) -> std::os::raw::c_long> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_STATUS_MALLOC:
    std::mem::MaybeUninit<extern fn(
        ptr: *mut *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_STATUS:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        status: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_STATUS_GET_AVAIL:
    std::mem::MaybeUninit<extern fn(
        obj: *const std::os::raw::c_void,
    ) -> std::os::raw::c_ulong> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_CLOSE:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_START:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_AVAIL_UPDATE:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_long> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_POLL_DESCRIPTORS_COUNT:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();
static mut FN_SND_PCM_POLL_DESCRIPTORS:
    std::mem::MaybeUninit<extern fn(
        pcm: *mut std::os::raw::c_void,
        pfds: *mut PollFd,
        space: std::os::raw::c_uint,
    ) -> std::os::raw::c_int> = std::mem::MaybeUninit::uninit();

static mut ALSA_DEVICE_INIT: Option<AlsaDevice> = None;

/// A module contains functions.
#[derive(Clone)]
pub struct AlsaDevice(std::marker::PhantomData<*mut u8>);

impl AlsaDevice {
    /// Get a handle to this module.  Loads module functions on first call.
    pub fn new() -> Option<Self> {
        unsafe {
            let dll = check_thread()?;
            if let Some(ref module) = ALSA_DEVICE_INIT {
                return Some(module.clone());
            }
            FN_SND_PCM_OPEN = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_open\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_MALLOC = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_malloc\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_ANY = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_any\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_SET_RATE_RESAMPLE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_set_rate_resample\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_SET_ACCESS = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_set_access\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_SET_FORMAT = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_set_format\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_SET_CHANNELS = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_set_channels\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_SET_RATE_NEAR = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_set_rate_near\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_GET_BUFFER_SIZE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_get_buffer_size\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_GET_PERIOD_SIZE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_get_period_size\0")?.as_ptr()));
            FN_SND_PCM_HW_PARAMS_FREE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_hw_params_free\0")?.as_ptr()));
            FN_SND_PCM_STATUS_MALLOC = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_status_malloc\0")?.as_ptr()));
            FN_SND_PCM_STATUS = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_status\0")?.as_ptr()));
            FN_SND_PCM_STATUS_GET_AVAIL = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_status_get_avail\0")?.as_ptr()));
            FN_SND_PCM_CLOSE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_close\0")?.as_ptr()));
            FN_SND_PCM_START = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_start\0")?.as_ptr()));
            FN_SND_PCM_AVAIL_UPDATE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_avail_update\0")?.as_ptr()));
            FN_SND_PCM_POLL_DESCRIPTORS_COUNT = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_poll_descriptors_count\0")?.as_ptr()));
            FN_SND_PCM_POLL_DESCRIPTORS = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_poll_descriptors\0")?.as_ptr()));
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
    pub fn snd_pcm_open(&self,
        name: &std::ffi::CStr,
        stream: SndPcmStream,
        mode: SndPcmMode,
    ) -> Result<SndPcm, i32>
    {
        unsafe {
            let mut pcmp = std::mem::MaybeUninit::uninit();
            let __ret = ((FN_SND_PCM_OPEN).assume_init())(
                pcmp.as_mut_ptr(),
                name.as_ptr(),
                stream as _,
                mode as _,
            );
            let pcmp = pcmp.assume_init();
            if __ret < 0 { return Err(__ret as _) };
            Ok(SndPcm(pcmp) as _)
        }
    }
    /// Allocate an invalid snd_pcm_hw_params_t using standard malloc
    pub fn snd_pcm_hw_params_malloc(&self,
    ) -> Result<SndPcmHwParams, i32>
    {
        unsafe {
            let mut ptr = std::mem::MaybeUninit::uninit();
            let __ret = ((FN_SND_PCM_HW_PARAMS_MALLOC).assume_init())(
                ptr.as_mut_ptr(),
            );
            let ptr = ptr.assume_init();
            if __ret < 0 { return Err(__ret as _) };
            Ok(SndPcmHwParams(ptr) as _)
        }
    }
    /// Fill params with a full configuration space for a PCM.
    pub fn snd_pcm_hw_params_any(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_ANY).assume_init())(
                pcm.0,
                params.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only real hardware rates.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space 
    /// - `val`: 0 = disable, 1 = enable (default) rate resampling
    pub fn snd_pcm_hw_params_set_rate_resample(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: u32,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_RATE_RESAMPLE).assume_init())(
                pcm.0,
                params.0,
                val as _,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only one access type.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space
    /// - `access`: access type
    pub fn snd_pcm_hw_params_set_access(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        access: SndPcmAccess,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_ACCESS).assume_init())(
                pcm.0,
                params.0,
                access as _,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only one format.
    pub fn snd_pcm_hw_params_set_format(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        format: SndPcmFormat,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_FORMAT).assume_init())(
                pcm.0,
                params.0,
                format as _,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Restrict a configuration space to contain only one channels count.
    pub fn snd_pcm_hw_params_set_channels(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: u32,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_CHANNELS).assume_init())(
                pcm.0,
                params.0,
                val as _,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Restrict a configuration space to have rate nearest to a target.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space
    /// - `val`: approximate target rate / returned approximate set rate
    /// - `dir`: Sub unit direction 
    pub fn snd_pcm_hw_params_set_rate_near(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
        val: &mut u32,
        dir: Option<&mut i32>,
    ) -> Result<(), i32>
    {
        unsafe {
            let mut __val: _ = *val as _;
            let mut __dir: _ = if let Some(_temp) = dir.iter().next() { Some(**_temp as _) } else { None };
            let __ret = ((FN_SND_PCM_HW_PARAMS_SET_RATE_NEAR).assume_init())(
                pcm.0,
                params.0,
                &mut __val,
                if let Some(ref mut _temp) = __dir { _temp } else { std::ptr::null_mut() },
            );
            *val = __val as _;
            if let Some(_temp) = dir { *_temp = __dir.unwrap() as _; }
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Install one PCM hardware configuration chosen from a configuration space
    /// and snd_pcm_prepare it.
    /// - `pcm`: PCM handle
    /// - `params`: Configuration space definition container
    pub fn snd_pcm_hw_params(&self,
        pcm: &SndPcm,
        params: &SndPcmHwParams,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_HW_PARAMS).assume_init())(
                pcm.0,
                params.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Extract buffer size from a configuration space.
    /// - `params`: Configuration space
    /// - `val`: Returned buffer size in frames
    pub fn snd_pcm_hw_params_get_buffer_size(&self,
        params: &SndPcmHwParams,
    ) -> Result<usize, i32>
    {
        unsafe {
            let mut val = std::mem::MaybeUninit::uninit();
            let __ret = ((FN_SND_PCM_HW_PARAMS_GET_BUFFER_SIZE).assume_init())(
                params.0,
                val.as_mut_ptr(),
            );
            let val = val.assume_init();
            if __ret < 0 { return Err(__ret as _) };
            Ok(val as _)
        }
    }
    /// Extract period size from a configuration space.
    /// - `params`: Configuration space
    /// - `val`: Returned approximate period size in frames
    /// - `dir`: Sub unit direction
    pub fn snd_pcm_hw_params_get_period_size(&self,
        params: &SndPcmHwParams,
        dir: Option<&mut i32>,
    ) -> Result<usize, i32>
    {
        unsafe {
            let mut frames = std::mem::MaybeUninit::uninit();
            let mut __dir: _ = if let Some(_temp) = dir.iter().next() { Some(**_temp as _) } else { None };
            let __ret = ((FN_SND_PCM_HW_PARAMS_GET_PERIOD_SIZE).assume_init())(
                params.0,
                frames.as_mut_ptr(),
                if let Some(ref mut _temp) = __dir { _temp } else { std::ptr::null_mut() },
            );
            let frames = frames.assume_init();
            if let Some(_temp) = dir { *_temp = __dir.unwrap() as _; }
            if __ret < 0 { return Err(__ret as _) };
            Ok(frames as _)
        }
    }
    /// Frees a previously allocated snd_pcm_hw_params_t
    pub fn snd_pcm_hw_params_free(&self,
        obj: &mut SndPcmHwParams,
    ) -> ()
    {
        unsafe {
            if obj.0.is_null() { panic!("Object free'd twice!") }
            let __ret = ((FN_SND_PCM_HW_PARAMS_FREE).assume_init())(
                obj.0,
            );
            obj.0 = std::ptr::null_mut();
            ()
        }
    }
    /// Obtain status (runtime) information for PCM handle. 
    /// - `pcm`: PCM handle
    /// - `status`: Status container
    pub fn snd_pcm_status_malloc(&self,
    ) -> Result<SndPcmStatus, i32>
    {
        unsafe {
            let mut ptr = std::mem::MaybeUninit::uninit();
            let __ret = ((FN_SND_PCM_STATUS_MALLOC).assume_init())(
                ptr.as_mut_ptr(),
            );
            let ptr = ptr.assume_init();
            if __ret < 0 { return Err(__ret as _) };
            Ok(SndPcmStatus(ptr) as _)
        }
    }
    /// Obtain status (runtime) information for PCM handle. 
    /// - `pcm`: PCM handle
    /// - `status`: Status container
    pub fn snd_pcm_status(&self,
        pcm: &SndPcm,
        status: &SndPcmStatus,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_STATUS).assume_init())(
                pcm.0,
                status.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Get number of frames available from a PCM status container (see
    /// snd_pcm_avail_update)
    pub fn snd_pcm_status_get_avail(&self,
        obj: &SndPcmStatus,
    ) -> usize
    {
        unsafe {
            let __ret = ((FN_SND_PCM_STATUS_GET_AVAIL).assume_init())(
                obj.0,
            );
            __ret as _
        }
    }
    /// close PCM handle
    pub fn snd_pcm_close(&self,
        pcm: &mut SndPcm,
    ) -> Result<(), i32>
    {
        unsafe {
            if pcm.0.is_null() { panic!("Object free'd twice!") }
            let __ret = ((FN_SND_PCM_CLOSE).assume_init())(
                pcm.0,
            );
            pcm.0 = std::ptr::null_mut();
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Start a PCM.
    pub fn snd_pcm_start(&self,
        pcm: &SndPcm,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_START).assume_init())(
                pcm.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
    /// Return number of frames ready to be read (capture) / written (playback)
    /// - `pcm`: PCM handle
    pub fn snd_pcm_avail_update(&self,
        pcm: &SndPcm,
    ) -> Result<isize, isize>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_AVAIL_UPDATE).assume_init())(
                pcm.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(__ret as _)
        }
    }
    /// Get count of poll descriptors for PCM handle
    pub fn snd_pcm_poll_descriptors_count(&self,
        pcm: &SndPcm,
    ) -> Result<i32, i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_POLL_DESCRIPTORS_COUNT).assume_init())(
                pcm.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(__ret as _)
        }
    }
    /// Get poll descriptors
    /// - `pcm`: PCM handle
    /// - `pfds`: Array of poll descriptors
    /// - `space`: space in the poll descriptor array 
    /// -Returns count of filled descriptors
    pub fn snd_pcm_poll_descriptors(&self,
        pcm: &SndPcm,
        pfds: &mut Vec<PollFd>,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_POLL_DESCRIPTORS).assume_init())(
                pcm.0,
                pfds.as_mut_ptr(),
                pfds.capacity() as _,
            );
            pfds.set_len(__ret as _);
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
}

static mut ALSA_PLAYER_INIT: Option<AlsaPlayer> = None;

/// A module contains functions.
#[derive(Clone)]
pub struct AlsaPlayer(std::marker::PhantomData<*mut u8>);

impl AlsaPlayer {
    /// Get a handle to this module.  Loads module functions on first call.
    pub fn new() -> Option<Self> {
        unsafe {
            let dll = check_thread()?;
            if let Some(ref module) = ALSA_PLAYER_INIT {
                return Some(module.clone());
            }
            FN_SND_PCM_PREPARE = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_prepare\0")?.as_ptr()));
            FN_SND_PCM_WRITEI = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_writei\0")?.as_ptr()));
            ALSA_PLAYER_INIT = Some(Self(std::marker::PhantomData));
            Some(Self(std::marker::PhantomData))
        }
    }
    /// Prepare PCM for use.
    pub fn snd_pcm_prepare(&self,
        pcm: &SndPcm,
    ) -> Result<(), i32>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_PREPARE).assume_init())(
                pcm.0,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
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
    pub fn snd_pcm_writei(&self,
        pcm: &SndPcm,
        buffer: &[u32],
    ) -> Result<isize, isize>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_WRITEI).assume_init())(
                pcm.0,
                buffer.as_ptr(),
                buffer.len() as _,
            );
            if __ret < 0 { return Err(__ret as _) };
            Ok(__ret as _)
        }
    }
}

static mut ALSA_RECORDER_INIT: Option<AlsaRecorder> = None;

/// A module contains functions.
#[derive(Clone)]
pub struct AlsaRecorder(std::marker::PhantomData<*mut u8>);

impl AlsaRecorder {
    /// Get a handle to this module.  Loads module functions on first call.
    pub fn new() -> Option<Self> {
        unsafe {
            let dll = check_thread()?;
            if let Some(ref module) = ALSA_RECORDER_INIT {
                return Some(module.clone());
            }
            FN_SND_PCM_READI = std::mem::MaybeUninit::new(std::mem::transmute(sym(dll, b"snd_pcm_readi\0")?.as_ptr()));
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
    pub fn snd_pcm_readi(&self,
        pcm: &SndPcm,
        buffer: &mut Vec<u32>,
    ) -> Result<(), isize>
    {
        unsafe {
            let __ret = ((FN_SND_PCM_READI).assume_init())(
                pcm.0,
                buffer.as_mut_ptr(),
                buffer.capacity() as _,
            );
            buffer.set_len(__ret as _);
            if __ret < 0 { return Err(__ret as _) };
            Ok(())
        }
    }
}
