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

use std::task::Context;
use std::ffi::CStr;
use super::{Native, NativeSpeakers, NativeMicrophone, NativeIterator};
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use std::mem::MaybeUninit;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::convert::TryInto;

/// Stream Mode
#[allow(unused)]
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
enum SndPcmMode {
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
enum SndPcmStream {
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
enum SndPcmAccess {
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
enum SndPcmFormat {
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
enum SndPcmState {
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
struct PollFd {
    fd: c_int,
    events: std::os::raw::c_short,
    revents: std::os::raw::c_short,
}

extern "C" {
    fn free(ptr: *mut c_void);
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

    // Poll
    fn snd_pcm_poll_descriptors(
        pcm: *mut c_void,
        pfds: *mut PollFd,
        space: c_uint
    ) -> c_int;
    fn snd_pcm_poll_descriptors_count(pcm: *mut c_void) -> c_int;

    // HW Params
    fn snd_pcm_hw_params(pcm: *mut c_void, params: *mut c_void) -> c_int;
    fn snd_pcm_hw_params_free(params: *mut c_void) -> ();
    fn snd_pcm_hw_params_set_rate_near(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
        dir: *mut c_int
    ) -> c_int;
    fn snd_pcm_hw_params_get_rate_numden(
        params: *mut c_void,
        rate_num: *mut c_uint,
        rate_den: *mut c_uint
    ) -> c_int;
    fn snd_pcm_hw_params_any(pcm: *mut c_void, params: *mut c_void) -> c_int;
    fn snd_pcm_hw_params_set_channels_max(
        pcm: *mut c_void,
        params: *mut c_void,
        val: *mut c_uint,
    ) -> c_int;
    fn snd_pcm_hw_params_set_channels(
        pcm: *mut c_void,
        params: *mut c_void,
        val: c_uint
    ) -> c_int;
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

/// Dynamically loaded library state (safe wrapper).
struct Lib(Alsa);

impl Lib {
    fn load() -> Option<Box<dyn Native>> {
        Some(Box::new(Self(
            Alsa::new().ok()?,
        )))
    }
}

impl Native for Lib {
    fn query_speakers(&self) -> NativeIterator<crate::Speakers> {
        let device_iter = DeviceIterator::<false>::new(self)
            .map(|dev| crate::Speakers::new(Speakers::with_device(dev)));
        NativeIterator(Box::new(std::iter::once(crate::Speakers::default()).chain(device_iter)))
    }

    fn query_microphones(&self) -> NativeIterator<crate::Microphone> {
        let device_iter = DeviceIterator::<true>::new(self)
            .map(|dev| crate::Microphone::new(Microphone::with_device(dev)));
        NativeIterator(Box::new(std::iter::once(crate::Microphone::default()).chain(device_iter)))
    }
}

struct Speakers {
    device: AudioDevice,
}

impl Speakers {
    fn with_device(device: AudioDevice) -> Self {
        Self {
            device,
        }
    }
}

impl Display for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Speakers")
    }
}

impl NativeSpeakers for Speakers {
    fn id(&self) -> &str {
        &self.device.name
    }

    fn is_ready(&mut self, cx: &mut Context<'_>) -> bool {
        false
    }

    fn play(&mut self, channels: usize) -> &mut [f32] {
        &mut []
    }
}

struct Microphone {
    device: AudioDevice,
}

impl Microphone {
    fn with_device(device: AudioDevice) -> Self {
        Self {
            device,
        }
    }
}

impl Display for Microphone {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Microphone")
    }
}

impl NativeMicrophone for Microphone {
    fn id(&self) -> &str {
        &self.device.name
    }

    fn is_ready(&mut self, cx: &mut Context<'_>) -> bool {
        false
    }

    fn record(&mut self, channels: usize) -> &[f32] {
        &[]
    }
}

// 
pub(super) fn init() -> Option<&'static mut dyn Native> {
    Lib::load()
}

/// Iterator over connected audio devices.
struct DeviceIterator<const M: bool> {
    hints: *mut *mut c_void,
    steps: *mut *mut c_void,
    drops: unsafe extern "C" fn(*mut *mut c_void) -> c_int,
    names: unsafe extern "C" fn(*const c_void, *const c_char) -> *mut c_char,
}

impl<const M: bool> DeviceIterator<M> {
    fn new(lib: &Lib) -> Self {
        let pcm = b"pcm\0";
        let mut hints = MaybeUninit::uninit();
        let hint = lib.0.snd_device_name_hint;
        let drops = lib.0.snd_device_name_free_hint;
        let names = lib.0.snd_device_name_get_hint;

        unsafe {
            if hint(-1, pcm.as_ptr().cast(), hints.as_mut_ptr()) < 0 {
                let hints = std::ptr::null_mut();
                let mut steps = std::ptr::null_mut();
                return DeviceIterator {
                    hints, steps, drops, names
                };
            }
            let hints = hints.assume_init();
            let mut steps = hints;
            DeviceIterator {
                hints, steps, drops, names
            }
        }
    }
}

impl<const M: bool> Iterator for DeviceIterator<M> {
    type Item = AudioDevice;
    
    fn next(&mut self) -> Option<Self::Item> {
        const NAME: &[u8] = b"NAME\0";
        const DESC: &[u8] = b"DESC\0";
        const IOID: &[u8] = b"IOID\0";

        unsafe {
            if self.steps.is_null() || (*self.steps).is_null() {
                return None;
            }

            // Allocate 3 C Strings describing device.
            let pcm_name = (self.names)(*self.steps, NAME.as_ptr().cast());
            let io = (self.names)(*self.steps, IOID.as_ptr().cast());
            debug_assert_ne!(pcm_name, std::ptr::null_mut());

            // Convert name to Rust String
            let id = CStr::from_ptr(pcm_name).to_str();
            // Filter out unused device names.
            if matches!(id, Ok("sysdefault") | Ok("null") | Ok("default")) {
                self.steps = self.steps.offset(1);
                return self.next();
            }

            // Convert description to Rust String
            let name = (self.names)(*self.steps, DESC.as_ptr().cast());
            assert_ne!(name, std::ptr::null_mut());
            let rust = CStr::from_ptr(name).to_string_lossy().to_string();
            free(name.cast());
            let name = rust.replace("\n", ": ");

            // Check device io direction.
            let is_input = io.is_null() || *(io.cast::<u8>()) == b'I';
            let is_output = io.is_null() || *(io.cast::<u8>()) == b'O';
            if !io.is_null() {
                free(io.cast());
            }

            // Right input type?
            let stream = if M { SndPcmStream::Capture } else { SndPcmStream::Playback };
            if (M && is_input) || (!M && is_output) {
                // Try to connect to PCM.
                let dev = connect_pcm(pcm_name, stream);

                if let Some((pcm, hwp, chans)) = dev {
                    // Add device to list of devices.
                    let fds = Vec::new();
                    free(pcm_name.cast());
                    self.steps = self.steps.offset(1);
                    return Some(AudioDevice {
                        name,
                        pcm,
                        hwp,
                        chans,
                        fds,
                    })
                }
            }

            free(pcm_name.cast());
            self.steps = self.steps.offset(1);
            self.next()
        }
    }
}

impl<const M: bool> Drop for DeviceIterator<M> {
    fn drop(&mut self) {
        if !self.hints.is_null() {
            unsafe {
                (self.drops)(self.hints);
            }
        }
    }
}

/// An Audio Device (input or output).
struct AudioDevice {
    /// Human-readable name for the device.
    name: String,
    /// PCM For Device.
    pcm: *mut c_void,
    /// Hardware parameters for device.
    hwp: *mut c_void,
    /// Number of channels
    chans: usize,
    /// File descriptors associated with this device.
    fds: Vec<smelling_salts::Device>,
}

impl AudioDevice {
    /// Generate file descriptors.
    fn start(&mut self) -> Option<()> {
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
            fd.old();
        }
        // Free hardware parameters and close PCM
        unsafe {
            pcm::hw_params_free(self.hwp);
            pcm::close(self.pcm).unwrap();
        }
    }
}

/// Open a PCM Device.
unsafe fn connect_pcm(
    alsa: &Alsa,
    name: *const c_char,
    stream: SndPcmStream,
) -> Option<(*mut c_void, *mut c_void, usize)> {
    let pcm = open(alsa, name, stream, SndPcmMode::Nonblock).ok()?;
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

unsafe fn open(
    alsa: &Alsa,
    name: *const c_char,
    stream: SndPcmStream,
    mode: SndPcmMode,
) -> Result<*mut c_void, i64> {
    let mut pcm = MaybeUninit::uninit();
    let ret =
        (alsa.snd_pcm_open)(pcm.as_mut_ptr(), name, stream, mode as c_int);
    let _: u64 = ret.try_into().map_err(|_| ret)?;
    let pcm = pcm.assume_init();
    Ok(pcm)
}

/// Get the number of channels of a PCM device.
unsafe fn configure_channels(alsa: &Alsa, device: &AudioDevice) -> usize {
    // Only allow up to 8 interleaved channels (what's supported by fon).
    let mut chans = 8;
    // Get the maximum number of channels supported by the hardware.
    (alsa.snd_pcm_hw_params_set_channels_max)(dev.pcm, dev.params, &mut chans);
    // Set the number of channels to the maximum.
    (alsa.snd_pcm_hw_params_set_channels)(dev.pcm, dev.params, chans);
    // Must be between 1 and 8.
    assert!(chans >= 1 && chans <= 8);
    // Return number of channels on this device.
    chans as usize
}

struct Global {
}

impl super::Global for Global {
    
}

#[inline(always)]
pub(super) fn global() -> Option<Box<dyn super::Global>> {
    // FIXME: Load shared object.

    // Return global state.
    Box::new(Global { })
}
