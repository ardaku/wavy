// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use super::{Playback, Recording};
use flume::Sender;
use smelling_salts::linux::{Device, Driver, RawDevice, Watcher};
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_long, c_uint, c_char, c_ulong, c_void};
use std::time::Duration;
use std::ffi::CStr;
use std::collections::HashSet;
use std::convert::TryInto;
use std::mem::MaybeUninit;

#[repr(C)]
struct TimeSpec {
    sec: isize,
    nsec: c_long,
}

#[repr(C)]
struct ITimerSpec {
    interval: TimeSpec,
    value: TimeSpec,
}

extern "C" {
    fn timerfd_create(clockid: c_int, flags: c_int) -> c_int;
    fn timerfd_settime(
        fd: c_int,
        flags: c_int,
        new_value: *const ITimerSpec,
        old_value: *mut ITimerSpec,
    ) -> c_int;
    fn read(fd: c_int, buf: *mut u64, count: usize) -> isize;
    fn close(fd: c_int) -> c_int;
    fn free(ptr: *mut c_void);
}



/// The environment-side speaker implementation
struct Speakers {
    fd: RawDevice,
    sender: Sender<Playback>,
}

/// The environment-side microphone implementation
struct Microphone {
    fd: RawDevice,
    sender: Sender<Recording>,
}

/// The environment-side listener implementation
struct Listener {
    // Set of currently discovered audio devices.
    discovered: HashSet<String>,
    // 0.5 second timer.
    timerfd: RawDevice,
    // 
    speaker_broadcaster: Sender<crate::Speakers>,
    // 
    microphone_broadcaster: Sender<crate::Microphone>,
}

impl Listener {
    fn new(
        speaker_broadcaster: Sender<crate::Speakers>,
        microphone_broadcaster: Sender<crate::Microphone>,
    ) -> Self {
        let discovered = HashSet::new();
        let dur = Duration::from_secs_f32(0.5);
        let sec = dur.as_secs() as _;
        let nsec = dur.subsec_nanos() as _;

        let timerfd = unsafe {
            timerfd_create(1 /*Monotonic*/, 2048 /*Nonblock*/)
        };
        let x = unsafe {
            timerfd_settime(
                timerfd,
                0,
                &ITimerSpec {
                    interval: TimeSpec { sec, nsec },
                    value: TimeSpec { sec, nsec },
                },
                std::ptr::null_mut(),
            )
        };
        assert_eq!(0, x);

        // 
        let mut listener = Self {
            discovered,
            timerfd,
            speaker_broadcaster,
            microphone_broadcaster,
        };

        // FIXME: Open default Speaker and Microphone first.
        println!("INPUT: Default");
        println!("OUTPUT: Default");

        // Check for non-default, non-null speakers and microphones
        // listener.react();
        listener
    }
}

/*
impl Reactor for Listener {
    fn raw(&self) -> RawDevice {
        self.timerfd
    }

    fn react(&mut self) {
        // Read out from timer file descriptor.
        let mut x = MaybeUninit::<u64>::zeroed();
        let v = unsafe {
            read(self.timerfd, x.as_mut_ptr(), std::mem::size_of::<u64>())
        };
        if v > 1 && unsafe { x.assume_init() } == 0 {
            return;
        }

        // 
        ALSA.with(|alsa| {
            let tpcm = CStr::from_bytes_with_nul(b"pcm\0").unwrap();
            let tname = CStr::from_bytes_with_nul(b"NAME\0").unwrap();
            let tdesc = CStr::from_bytes_with_nul(b"DESC\0").unwrap();
            let tioid = CStr::from_bytes_with_nul(b"IOID\0").unwrap();

            let mut hints = MaybeUninit::uninit();
            unsafe {
                if (alsa.snd_device_name_hint)(-1, tpcm.as_ptr(), hints.as_mut_ptr())
                    < 0
                {
                    return;
                }
                let hints = hints.assume_init();
                let mut n = hints;
                while !(*n).is_null() {
                    // Allocate 3 C Strings describing device.
                    let pcm_name = (alsa.snd_device_name_get_hint)(*n, tname.as_ptr());
                    let io = (alsa.snd_device_name_get_hint)(*n, tioid.as_ptr());
                    debug_assert_ne!(pcm_name, std::ptr::null_mut());

                    // Convert description to Rust String
                    let name = match CStr::from_ptr(pcm_name).to_str() {
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
                            if self.discovered.insert(hwid.to_string()) {
                                let name =
                                    (alsa.snd_device_name_get_hint)(*n, tdesc.as_ptr());
                                assert_ne!(name, std::ptr::null_mut());
                                let rust =
                                    CStr::from_ptr(name).to_string_lossy().to_string();
                                free(name.cast());
                                Some(rust.replace("\n", ": "))
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
                    if let Some(name) = name {
                        if is_input {
                            if let Some(pcm) = open(alsa, pcm_name, SndPcmStream::Capture) {
                                println!("INPUT: {}", name);
                            }
                        }
                        if is_output {
                            if let Some(pcm) = open(alsa, pcm_name, SndPcmStream::Playback) {
                                println!("OUTPUT: {}", name);
                            }
                        }
                    }

                    /*if (D::INPUT && is_input) || (!D::INPUT && is_output) {
                        // Try to connect to PCM.
                        let dev = open(
                            pcm_name,
                            if D::INPUT {
                                SndPcmStream::Capture
                            } else {
                                SndPcmStream::Playback
                            },
                        );

                        if let Some((pcm, hwp, supported)) = dev {
                            // Add device to list of devices.
                            devices.push(abstrakt(D::from(AudioDevice {
                                name,
                                pcm,
                                hwp,
                                supported,
                                fds: Vec::new(),
                            })));
                        }
                    }*/
                    free(pcm_name.cast());
                    n = n.offset(1);
                }
                (alsa.snd_device_name_free_hint)(hints);
            }
        });

        // FIXME: receivers
        /*speaker_broadcaster.send(crate::Speakers::new(receiver));
        microphone_broadcaster.sender(crate::Microphone::new(receiver));*/
    }

    fn drop(&mut self) {
        assert_eq!(0, unsafe { close(self.timerfd) });
    }
}*/

/*// The epoll loop that handles finding & comms w/ speakers and microphones
fn audio_task(listener: Listener) {
    // Environment-side implementation for speakers and microphones
    let mut speakers = Vec::<(Vec<u8>, Speakers)>::new();
    let mut mics = Vec::<(Vec<u8>, Microphone)>::new();

    // Application-side currently unused speakers and microphones
    let mut app_speakers = Vec::<crate::Speakers>::new();
    let mut app_mics = Vec::<crate::Microphone>::new();

    // Create the smelling salts `Sleeper`
    let sleeper = Sleeper::new();

    // Add Speaker/Microphone PCM listener, checking for new devices at 0.5
    // second intervals.
    let listener = sleeper.watch(listener, Watcher::new().input());

    // Put this thread to sleep, and call the reactor callbacks when woken.
    sleeper.sleep();

    /*
        match event_type {

            // This event is triggered every time a speaker is ready
            1 /* SPEAKER(index) */ => {
                // Non-blocking (lossy) send of writeable audio buffer.

                // FIXME
                let playback = Playback {
                    callback: fn(&mut dyn Iterator<Item = Frame<Ch32, 8>>),
                };
                if speakers[index].sender.try_send(playback).is_err() {
                    // FIXME: Suppress debug print
                    eprintln!("Application audio playback is too slow!");
                }
            },

            // This event is triggered every time a microphone is ready.
            2 /* MICROPHONE(index) */ => {
                // Non-blocking (lossy) send of readable audio buffer.

                // FIXME
                let recording = Recording {
                    callback: fn(&mut dyn FnMut(Frame<Ch32, 8>)),
                };
                if microphones[index].sender.try_send(recording).is_err() {
                    // FIXME: Suppress debug print
                    eprintln!("Application audio recording is too slow!");
                }
            },
            _ => unreachable!(),
        }
    }*/
}*/

/// Start the audio thread for the Linux platform
pub(super) fn start(
    speaker_broadcaster: Sender<crate::Speakers>,
    microphone_broadcaster: Sender<crate::Microphone>,
) {
    std::thread::spawn(|| {
//        audio_task(Listener::new(speaker_broadcaster, microphone_broadcaster))
    });
}

// Safe-ish open wrapper for non-blocking PCMs.
unsafe fn open(
    alsa: &Alsa,
    name: *const c_char,
    stream: SndPcmStream,
) -> Option<*mut c_void> {
    // Create the PCM.
    let mut pcm = MaybeUninit::uninit();
    let _: u64 = (alsa.snd_pcm_open)(pcm.as_mut_ptr(), name, stream, SndPcmMode::Nonblock as c_int).try_into().ok()?;
    let pcm = pcm.assume_init();

    // Allocate Hardware Parameters To Configure the PCM with.
    let mut hwp = MaybeUninit::uninit();
    let _: u64 = (alsa.snd_pcm_hw_params_malloc)(hwp.as_mut_ptr()).try_into().ok()?;
    let hwp = hwp.assume_init();

    // Let the Linux kernel choose default settings.
    let _: u64 = (alsa.snd_pcm_hw_params_any)(pcm, hwp).try_into().ok()?;

    // Set the audio format (i16 - LE), and access (mmap interleaved).  This is
    // necessary to support as many devices as possible.
    let _: u64 = (alsa.snd_pcm_hw_params_set_access)(pcm, hwp, SndPcmAccess::MmapInterleaved).try_into().ok()?;
    let _: u64 = (alsa.snd_pcm_hw_params_set_format)(pcm, hwp, SndPcmFormat::S16Le).try_into().ok()?;

    // Return the pcm
    Some(pcm)
}
