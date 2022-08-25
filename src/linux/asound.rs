// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! FFI bindings to libasound

#![allow(unsafe_code)]

use std::os::raw::{c_int, c_uint, c_void, c_long, c_ulong, c_char, c_short};
use std::mem::MaybeUninit;
use std::ffi::CStr;
use std::collections::HashSet;

use fon::chan::Channel;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct SndPcmArea {
    /// Base address of channel samples
    addr: *mut c_void,
    /// Offset to first sample in bits 
    first: c_uint,
    /// Samples distance in bits 
    step: c_uint,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(super) struct PollFd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

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

/// Supported audio formats for Linux driver for Wavy.
#[derive(Debug)]
enum AudioFormat {
    /// 32-bit floating point audio preferred.
    Float32,
    /// 32-bit integer next (f32 -1:1 <-> Ch24+Ch8 -MIN:MAX)
    Int32,
    /// 24-bit integer after (f32 -1:1 <-> Ch24 -MIN:MAX)
    Int24,
    /// Fallback: most devices support 16-bit audio
    Int16,
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
    fn snd_pcm_mmap_begin(
        pcm: *mut c_void,
        areas: *mut *const SndPcmArea,
        offset: *mut c_ulong,
        frames: *mut c_ulong,
    ) -> c_int;
    fn snd_pcm_mmap_commit(
        pcm: *mut c_void,
        offset: c_ulong,
        frames: c_ulong,
    ) -> c_ulong;
    fn snd_pcm_avail_update(pcm: *mut c_void) -> c_long;

    // Poll
    fn snd_pcm_poll_descriptors(
        pcm: *mut c_void,
        pfds: *mut PollFd,
        space: c_uint,
    ) -> c_int;
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

extern "C" {
    fn close(fd: c_int) -> c_int;
    fn free(ptr: *mut c_void);
}

thread_local! {
    static ALSA: Alsa = Alsa::new().expect("Error: Linux without ALSA!");
}

/// Type safe PCM.
struct Pcm(*mut c_void);

impl Drop for Pcm {
    fn drop(&mut self) {
        ALSA.with(|alsa| unsafe { (alsa.snd_pcm_close)(self.0) });
    }
}

struct Hwp(*mut c_void);

impl Drop for Hwp {
    fn drop(&mut self) {
        ALSA.with(|alsa| unsafe { (alsa.snd_pcm_hw_params_free)(self.0) });
    }
}

/// Shared code between ALSA speaker and ALSA microphone.
pub(crate) struct AudioDevice {
    pcm: Pcm,
    fmt: AudioFormat,
    pub(super) name: String,
}

// Safe-ish open wrapper for non-blocking PCMs.
unsafe fn open(
    alsa: &Alsa,
    name: *const c_char,
    stream: SndPcmStream,
) -> Option<(Pcm, AudioFormat)> {
    // Create the PCM.
    let mut pcm = MaybeUninit::uninit();
    let _: u64 = (alsa.snd_pcm_open)(pcm.as_mut_ptr(), name, stream, SndPcmMode::Nonblock as c_int).try_into().ok()?;
    let pcm = Pcm(pcm.assume_init());

    // Allocate Hardware Parameters To Configure the PCM with.
    let mut hwp = MaybeUninit::uninit();
    let _: u64 = (alsa.snd_pcm_hw_params_malloc)(hwp.as_mut_ptr()).try_into().ok()?;
    let hwp = Hwp(hwp.assume_init());

    // Let the Linux kernel choose default settings.
    let _: u64 = (alsa.snd_pcm_hw_params_any)(pcm.0, hwp.0).try_into().ok()?;

    // Set the audio access (mmap interleaved - supports more devices than
    // non-interleaved).
    let _: u64 = (alsa.snd_pcm_hw_params_set_access)(pcm.0, hwp.0, SndPcmAccess::MmapInterleaved).try_into().ok()?;

    // Set audio format (Try: f32, i24, i16 - LE)
    if u64::try_from((alsa.snd_pcm_hw_params_set_format)(pcm.0, hwp.0, SndPcmFormat::FloatLe)).is_ok() {
        Some((pcm, AudioFormat::Float32))
    } else if u64::try_from((alsa.snd_pcm_hw_params_set_format)(pcm.0, hwp.0, SndPcmFormat::S32Le)).is_ok() {
        Some((pcm, AudioFormat::Int32))
    } else if u64::try_from((alsa.snd_pcm_hw_params_set_format)(pcm.0, hwp.0, SndPcmFormat::S24Le)).is_ok() {
        Some((pcm, AudioFormat::Int24))
    } else {
        u64::try_from((alsa.snd_pcm_hw_params_set_format)(pcm.0, hwp.0, SndPcmFormat::S16Le)).ok()?;
        Some((pcm, AudioFormat::Int16))
    }
}

impl Alsa {
    fn devices(&self, direction: SndPcmStream, list: &mut HashSet<String>) -> Vec<AudioDevice> {
        let mut devices = Vec::new();

        unsafe {
            if list.insert("default".to_string()) {
                let pcm_name: *const u8 = b"default\0".as_ptr();
                let pcm_name: *const c_char = pcm_name.cast();
                if let Some((pcm, fmt)) = open(self, pcm_name, direction) {
                    let name = "Default (default)".to_string();
                    devices.push(AudioDevice { pcm, fmt, name });
                }
            }
        }
    
        let tpcm = CStr::from_bytes_with_nul(b"pcm\0").unwrap();
        let tname = CStr::from_bytes_with_nul(b"NAME\0").unwrap();
        let tdesc = CStr::from_bytes_with_nul(b"DESC\0").unwrap();
        let tioid = CStr::from_bytes_with_nul(b"IOID\0").unwrap();

        let mut hints = MaybeUninit::uninit();
        unsafe {
            if (self.snd_device_name_hint)(-1, tpcm.as_ptr(), hints.as_mut_ptr())
                < 0
            {
                return devices;
            }
            let hints = hints.assume_init();
            let mut n = hints;
            while !(*n).is_null() {
                // Allocate 3 C Strings describing device.
                let pcm_name = (self.snd_device_name_get_hint)(*n, tname.as_ptr());
                let io = (self.snd_device_name_get_hint)(*n, tioid.as_ptr());
                debug_assert_ne!(pcm_name, std::ptr::null_mut());

                // Convert description to Rust String
                let mut name = match CStr::from_ptr(pcm_name).to_str() {
                    Err(_) => {
                        n = n.offset(1);
                        continue;
                    }
                    Ok(x) if x.starts_with("sysdefault") => {
                        n = n.offset(1);
                        continue;
                    }
                    Ok("null") | Ok("default") => {
                        n = n.offset(1);
                        continue;
                    }
                    Ok(hwid) => {
                        // Add hardware ID to the discovered HashSet
                        if list.insert(hwid.to_string()) {
                            let name =
                                (self.snd_device_name_get_hint)(*n, tdesc.as_ptr());
                            assert_ne!(name, std::ptr::null_mut());
                            let rust =
                                CStr::from_ptr(name).to_string_lossy().to_string();
                            free(name.cast());
                            rust.lines().next().map(|x| format!("{} ({})", x, hwid))
                        } else {
                            None
                        }
                    }
                };

                // Check device io direction.
                let is_input = io.is_null() || *(io.cast::<u8>()) == b'I';
                let is_output = io.is_null() || *(io.cast::<u8>()) == b'O';
                if !io.is_null() {
                    free(io.cast());
                }

                // Right input type?
                if let Some(name) = name.take() {
                    if is_input && direction == SndPcmStream::Capture {
                        if let Some((pcm, fmt)) = open(self, pcm_name, direction) {
                            devices.push(AudioDevice {
                                pcm, fmt, name
                            });
                        }
                    } else if is_output && direction == SndPcmStream::Playback {
                        if let Some((pcm, fmt)) = open(self, pcm_name, direction) {
                            devices.push(AudioDevice {
                                pcm, fmt, name
                            });
                        }
                    }
                }

                free(pcm_name.cast());
                n = n.offset(1);
            }
            (self.snd_device_name_free_hint)(hints);
        }

        devices.reverse();
        devices
    }
}

impl AudioDevice {
    unsafe fn play_alsa(&self, alsa: &Alsa, output: &mut Vec<f32>) {
        // Setup
        let mut areas = MaybeUninit::<*const SndPcmArea>::uninit();
        let mut offset = MaybeUninit::<c_ulong>::uninit();
        let mut frames = MaybeUninit::<c_ulong>::uninit();
        let frames_ready = (alsa.snd_pcm_avail_update)(self.pcm.0);
        let ret = (alsa.snd_pcm_mmap_begin)(self.pcm.0, areas.as_mut_ptr(), offset.as_mut_ptr(), frames.as_mut_ptr());
        debug_assert_eq!(0, ret);
        let offset = offset.assume_init();
        let frames = frames.assume_init();
        let frame_count: usize = frames.try_into().unwrap();
        let areas = areas.assume_init();
        let area = (*areas).addr;
        dbg!(*areas);
        dbg!(frames_ready, frame_count); // equal?

        // 
        output.clear();
        match self.fmt {
            AudioFormat::Float32 => {
                let mut area: *mut f32 = area.cast();
                for sample in output.drain(..frame_count) {
                    *area = sample;
                    area = area.offset(1);
                }
            },
            AudioFormat::Int32 => {
                let mut area: *mut i32 = area.cast();
                for sample in output.drain(..frame_count) {
                    *area = (i32::from(fon::chan::Ch24::from(sample)) << 8) + 127;
                    area = area.offset(1);
                }
            },
            AudioFormat::Int24 => {
                let mut area: *mut i32 = area.cast();
                for sample in output.drain(..frame_count) {
                    *area = fon::chan::Ch24::from(sample).into();
                    area = area.offset(1);
                }
            },
            AudioFormat::Int16 => {
                let mut area: *mut i16 = area.cast();            
                for sample in output.drain(..frame_count) {
                    *area = fon::chan::Ch16::from(sample).into();
                     area = area.offset(1);
                }
            },
        }

        // Commit
        let ret = (alsa.snd_pcm_mmap_commit)(self.pcm.0, offset, frames);
        debug_assert_eq!(frames, ret);
    }
    
    unsafe fn record_alsa(&self, alsa: &Alsa, output: &mut Vec<f32>) {
        // Setup
        let mut areas = MaybeUninit::<*const SndPcmArea>::uninit();
        let mut offset = MaybeUninit::<c_ulong>::uninit();
        let mut frames = MaybeUninit::<c_ulong>::uninit();
        let frames_ready = (alsa.snd_pcm_avail_update)(self.pcm.0);
        let ret = (alsa.snd_pcm_mmap_begin)(self.pcm.0, areas.as_mut_ptr(), offset.as_mut_ptr(), frames.as_mut_ptr());
        debug_assert_eq!(0, ret);
        let offset = offset.assume_init();
        let frames = frames.assume_init();
        let frame_count: usize = frames.try_into().unwrap();
        let areas = areas.assume_init();
        let area = (*areas).addr;
        dbg!(*areas);
        dbg!(frames_ready, frame_count);

        // 
        output.clear();
        match self.fmt {
            AudioFormat::Float32 => {
                let mut area: *mut f32 = area.cast();
                for _ in 0..frame_count {
                    let sample: f32 = *area;
                    output.push(sample);
                    area = area.offset(1);
                }
            },
            AudioFormat::Int32 => {
                let mut area: *mut i32 = area.cast();
                for _ in 0..frame_count {
                    let sample: i32 = *area;
                    output.push(fon::chan::Ch24::new(sample >> 8).to_f32());
                    area = area.offset(1);
                }
            },
            AudioFormat::Int24 => {
                let mut area: *mut i32 = area.cast();
                for _ in 0..frame_count {
                    let sample: i32 = *area;
                    output.push(fon::chan::Ch24::new(sample).to_f32());
                    area = area.offset(1);
                }
            },
            AudioFormat::Int16 => {
                let mut area: *mut i16 = area.cast();
                for _ in 0..frame_count {
                    let sample: i16 = *area;
                    output.push(fon::chan::Ch16::new(sample).to_f32());
                    area = area.offset(1);
                }
            },
        }

        // Commit
        let ret = (alsa.snd_pcm_mmap_commit)(self.pcm.0, offset, frames);
        debug_assert_eq!(frames, ret);
    }

    pub(super) fn record(&self, output: &mut Vec<f32>) {
        unsafe {
            ALSA.with(|alsa| self.record_alsa(alsa, output));
        }
    }

    pub(super) fn play(&self, output: &mut Vec<f32>) {
        unsafe {
            ALSA.with(|alsa| self.play_alsa(alsa, output));
        }
    }
}

pub(super) fn devices(direction: SndPcmStream, list: &mut HashSet<String>) -> Vec<AudioDevice> {
    ALSA.with(|alsa| alsa.devices(direction, list))
}
