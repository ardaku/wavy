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

use std::os::raw::c_void;
use std::sync::Arc;

use crate::consts::SAMPLE_RATE;

#[repr(C)]
pub(crate) struct AudioStreamBasicDescription {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

impl AudioStreamBasicDescription {
    pub(crate) const fn new(channels: u32) -> Self {
        Self {
            // Use wavy's target sample rate
            sample_rate: SAMPLE_RATE as f64,
            // Don't do any compression, use linear PCM (raw samples).
            format_id: u32::from_ne_bytes(*b"lpcm"),
            // Set flag for float (if unset, integer).
            format_flags: 1,
            // sizeof(f32) * channels
            bytes_per_packet: 4 * channels,
            // Always one
            frames_per_packet: 1,
            // Same as bytes_per_packet
            bytes_per_frame: 4 * channels,
            // Stereo (channels)
            channels_per_frame: channels,
            // sizeof(f32) * bits_in_a_byte
            bits_per_channel: 4 * 8,
            // Always zero
            reserved: 0,
        }
    }
}

// MacOS RunLoop is thread-safe.
#[repr(transparent)]
struct RunLoop(*mut c_void);

unsafe impl Send for RunLoop {}
unsafe impl Sync for RunLoop {}

extern "C" {
    fn CFRunLoopGetCurrent() -> RunLoop;
    fn CFRunLoopRun();
}

// Thread-local run loop for audio device (global state).
thread_local!(static RUN_LOOP: RunLoop = initialize());

/// Global initialization, may be called more than once.
fn initialize() -> RunLoop {
    let pair = std::sync::Arc::new((std::sync::Mutex::new(None), std::sync::Condvar::new()));
    let pair2 = Arc::clone(&pair);
    std::thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        // Get the run loop associated with this thread.
        let run_loop = unsafe { CFRunLoopGetCurrent() };
        *started = Some(run_loop);
        cvar.notify_one();
        // Block this thread with the run loop until the process exits.
        unsafe { CFRunLoopRun() }
    });

    // Wait to get the thread's Run Loop handle.
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    let run_loop = loop {
        if let Some(run_loop) = started.take() {
            break run_loop;
        }
        started = cvar.wait(started).unwrap();
    };
    run_loop
}
