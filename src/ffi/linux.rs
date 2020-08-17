// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use self::gen::{
    AlsaDevice, AlsaPlayer, AlsaRecorder, SndPcm, SndPcmAccess, SndPcmFormat,
    SndPcmHwParams, SndPcmMode, SndPcmState, SndPcmStream,
};
use crate::frame::Frame;
use fon::{chan::Ch16, sample::Sample, stereo::Stereo16, Audio};
use std::{
    any::TypeId,
    convert::TryInto,
    ffi::CString,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

// Update with: `dl_api ffi/asound,so,2.muon src/linux/gen.rs`
#[path = "alsa/gen.rs"]
mod gen;

fn pcm_hw_params(
    device: &AlsaDevice,
    sound_device: &SndPcm,
    limit_buffer: bool,
) -> Option<(SndPcmHwParams, u32)> {
    // unwrap: Allocating memory should not fail unless out of memory.
    let hw_params = device.snd_pcm_hw_params_malloc().unwrap();
    // unwrap: Getting default settings should never fail.
    device
        .snd_pcm_hw_params_any(sound_device, &hw_params)
        .unwrap();
    // Tell it not to resample
    device
        .snd_pcm_hw_params_set_rate_resample(sound_device, &hw_params, 0)
        .unwrap();
    // Find something near 48_000 (this will prefer 48_000, but fallback to
    // 44_100, if that's what's supported.
    let mut actual_rate = 48_000;
    device
        .snd_pcm_hw_params_set_rate_near(
            sound_device,
            &hw_params,
            &mut actual_rate,
            None,
        )
        .unwrap();
    // Set access to RW noninterleaved.
    // unwrap: Kernel should always support interleaved mode.
    device
        .snd_pcm_hw_params_set_access(
            sound_device,
            &hw_params,
            SndPcmAccess::RwInterleaved,
        )
        .unwrap();
    // unwrap: S16LE should be supported by all popular speakers / microphones.
    device
        .snd_pcm_hw_params_set_format(
            sound_device,
            &hw_params,
            SndPcmFormat::S16Le,
        )
        .expect("PCM does not support S16LE");
    // Set channels to stereo (2).
    // unwrap: FIXME: Fallback SIMD resampler if this were to fail.
    device
        .snd_pcm_hw_params_set_channels(sound_device, &hw_params, 2)
        .unwrap();
    // Period size must be a power of two
    // Currently only tries 1024
    let mut period_size = 1024;
    device
        .snd_pcm_hw_params_set_period_size_near(
            sound_device,
            &hw_params,
            &mut period_size,
            None,
        )
        .unwrap();
    if period_size != 1024 {
        panic!(
            "Wavy: Tried to set period size: {}, Got: {}!",
            1024, period_size
        );
    }
    // Set buffer size to about 3 times the period (setting latency).
    if limit_buffer {
        let mut buffer_size = 1024 * 3;
        // unwrap: Some buffer size should always be available.
        device
            .snd_pcm_hw_params_set_buffer_size_near(
                sound_device,
                &hw_params,
                &mut buffer_size,
            )
            .unwrap();
    } else {
        // Apply the hardware parameters that just got set.
        // unwrap: Should always be able to apply parameters that succeeded
        device.snd_pcm_hw_params(sound_device, &hw_params).unwrap();
        // Get rid of garbage.
        // unwrap: Should always be able free data from the heap.
        device.snd_pcm_drop(&sound_device).unwrap();
    }
    // Re-Apply the hardware parameters that just got set.
    // unwrap: Should always be able to apply parameters that succeeded
    device.snd_pcm_hw_params(sound_device, &hw_params).unwrap();

    Some((hw_params, actual_rate))
}

// Player/Recorder Shared Code for ALSA.
pub(super) struct Pcm {
    device: AlsaDevice,
    sound_device: SndPcm,
    fd: smelling_salts::Device,
}

impl Pcm {
    /// Create a new async PCM.
    fn new(direction: SndPcmStream) -> Option<(Self, u32)> {
        // Load shared alsa module.
        let device = AlsaDevice::new()?;
        // FIXME: Currently only the default device is supported.
        let device_name = CString::new("default").unwrap();
        // Create the ALSA PCM.
        let sound_device: SndPcm = device
            .snd_pcm_open(&device_name, direction, SndPcmMode::Nonblock)
            .ok()?;
        // Configure Hardware Parameters
        let (mut hw_params, sample_rate) = pcm_hw_params(
            &device,
            &sound_device,
            direction == SndPcmStream::Playback,
        )?;
        // Free Hardware Parameters
        device.snd_pcm_hw_params_free(&mut hw_params);
        // Get file descriptor
        let fd_count = device
            .snd_pcm_poll_descriptors_count(&sound_device)
            .unwrap();
        let mut fd_list = Vec::with_capacity(fd_count.try_into().unwrap());
        device
            .snd_pcm_poll_descriptors(&sound_device, &mut fd_list)
            .unwrap();
        // FIXME: More?
        assert_eq!(fd_count, 1);
        // Register file descriptor with OS's I/O Event Notifier
        let fd = smelling_salts::Device::new(
            fd_list[0].fd,
            #[allow(unsafe_code)]
            unsafe {
                smelling_salts::Watcher::from_raw(fd_list[0].events as u32)
            },
        );

        Some((
            Pcm {
                device,
                sound_device,
                fd,
            },
            sample_rate,
        ))
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        // Unregister async file descriptor before closing the PCM.
        self.fd.old();
        // Should never fail here
        self.device.snd_pcm_close(&mut self.sound_device).unwrap();
    }
}

pub(crate) struct Speakers<S: Sample>
where
    Ch16: From<S::Chan>,
{
    player: AlsaPlayer,
    pcm: Pcm,
    is_ready: bool,
    _phantom: PhantomData<S>,
}

impl<S: Sample + Unpin> Future for &mut Speakers<S>
where
    Ch16: From<S::Chan>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_ready {
            // Default to ready
            Poll::Ready(())
        } else {
            let this = self.get_mut();
            // Next time this task wakes up, assume ready.
            this.is_ready = true;
            // Register waker, and then return not ready.
            this.pcm.fd.register_waker(cx.waker());
            Poll::Pending
        }
    }
}

impl<S: Sample> Speakers<S>
where
    Ch16: From<S::Chan>,
{
    pub(crate) fn connect() -> (Self, u32) {
        // Load Player ALSA module
        // unwrap(): FIXME - Should connect to dummy backend instead if fail.
        let player = AlsaPlayer::new().unwrap();
        // Create Playback PCM.
        // unwrap(): FIXME - Should connect to dummy backend instead if fail.
        let (pcm, sample_rate) = Pcm::new(SndPcmStream::Playback).unwrap();
        let is_ready = true;
        let _phantom = PhantomData;

        (
            Self {
                player,
                pcm,
                is_ready,
                _phantom,
            },
            sample_rate,
        )
    }

    #[allow(unsafe_code)]
    pub(crate) fn play(&mut self, audio: &Audio<S>) -> usize {
        const SILENCE: [[u8; 4]; 1024] = [[0; 4]; 1024];

        // Record into temporary buffer.
        let ret = if audio.is_empty() {
            // Play 1024 samples of silence
            unsafe {
                self.player.snd_pcm_writei(
                    &self.pcm.sound_device,
                    SILENCE.as_ptr().cast(),
                    SILENCE.len(),
                )
            }
        } else if TypeId::of::<Stereo16>() == TypeId::of::<S>() {
            // Play unconverted slice of audio.
            unsafe {
                self.player.snd_pcm_writei(
                    &self.pcm.sound_device,
                    audio.as_u8_slice().as_ptr().cast(),
                    audio.len(),
                )
            }
        } else {
            // Play converted slice of audio.
            Audio::<Stereo16>::with_audio(audio.sample_rate(), audio);
            unsafe {
                self.player.snd_pcm_writei(
                    &self.pcm.sound_device,
                    audio.as_u8_slice().as_ptr().cast(),
                    audio.len(),
                )
            }
        };
        match ret {
            Err(error) => {
                let state =
                    self.pcm.device.snd_pcm_state(&self.pcm.sound_device);
                match error {
                    // Edge-triggered epoll should only go into pending mode if
                    // read/write call results in EAGAIN (according to epoll man
                    // page)
                    -11 => {
                        self.is_ready = false;
                        0
                    }
                    -32 => match state {
                        SndPcmState::Xrun => {
                            // Player samples are not generated fast enough
                            self.pcm
                                .device
                                .snd_pcm_prepare(&self.pcm.sound_device)
                                .unwrap();
                            unsafe {
                                self.player
                                    .snd_pcm_writei(
                                        &self.pcm.sound_device,
                                        SILENCE.as_ptr().cast(),
                                        SILENCE.len(),
                                    )
                                    .unwrap();
                            }
                            0
                        }
                        st => {
                            eprintln!("Incorrect state = {:?} (XRUN): Report Bug to https://github.com/libcala/wavy/issues/new", st);
                            unreachable!()
                        }
                    },
                    -77 => {
                        eprintln!("Incorrect state (-EBADFD): Report Bug to https://github.com/libcala/wavy/issues/new");
                        unreachable!()
                    }
                    -86 => {
                        eprintln!("Stream got suspended, trying to recover… (-ESTRPIPE)");
                        if self
                            .pcm
                            .device
                            .snd_pcm_resume(&self.pcm.sound_device)
                            .is_ok()
                        {
                            // Prepare, so we keep getting samples.
                            self.pcm
                                .device
                                .snd_pcm_prepare(&self.pcm.sound_device)
                                .unwrap();
                            unsafe {
                                self.player
                                    .snd_pcm_writei(
                                        &self.pcm.sound_device,
                                        SILENCE.as_ptr().cast(),
                                        SILENCE.len(),
                                    )
                                    .unwrap();
                            }
                        }
                        0
                    }
                    _ => unreachable!(),
                }
            }
            Ok(len) => len as usize,
        }
    }
}

pub(crate) struct Recorder<F: Frame> {
    recorder: AlsaRecorder,
    pcm: Pcm,
    sample_rate: u32,
    is_ready: bool,
    _phantom: PhantomData<F>,
}

impl<F: Frame> Recorder<F> {
    pub(crate) fn new() -> Option<Self> {
        // Load Recorder ALSA module
        let recorder = AlsaRecorder::new()?;
        // Create Capture PCM.
        let (pcm, sample_rate) = Pcm::new(SndPcmStream::Capture)?;
        // pcm.device.snd_pcm_start(&pcm.sound_device).unwrap();
        let is_ready = true;
        let _phantom = PhantomData;
        // Return successfully
        Some(Recorder {
            recorder,
            pcm,
            sample_rate,
            is_ready,
            _phantom,
        })
    }

    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<u32> {
        if self.is_ready {
            // Default to ready
            Poll::Ready(self.sample_rate)
        } else {
            // Next time this task wakes up, assume ready.
            self.is_ready = true;
            // Register waker, and then return not ready.
            self.pcm.fd.register_waker(cx.waker());
            Poll::Pending
        }
    }

    pub(crate) fn record_last(&mut self, audio: &mut Vec<F>) {
        // Record into temporary buffer.
        if let Err(error) =
            self.recorder.snd_pcm_readi(&self.pcm.sound_device, audio)
        {
            let state = self.pcm.device.snd_pcm_state(&self.pcm.sound_device);
            match error {
                // Edge-triggered epoll should only go into pending mode if
                // read/write call results in EAGAIN (according to epoll man
                // page)
                -11 => {
                    self.is_ready = false;
                }
                -77 => {
                    eprintln!("Incorrect state (-EBADFD): Report Bug to https://github.com/libcala/wavy/issues/new");
                    unreachable!()
                }
                -32 => match state {
                    SndPcmState::Xrun => {
                        eprintln!("Recorder XRUN: Latency cause?");
                        self.pcm
                            .device
                            .snd_pcm_prepare(&self.pcm.sound_device)
                            .unwrap();
                    }
                    st => {
                        eprintln!("Incorrect state = {:?} (XRUN): Report Bug to https://github.com/libcala/wavy/issues/new", st);
                        unreachable!()
                    }
                },
                -86 => {
                    eprintln!(
                        "Stream got suspended, trying to recover… (-ESTRPIPE)"
                    );
                    if self
                        .pcm
                        .device
                        .snd_pcm_resume(&self.pcm.sound_device)
                        .is_ok()
                    {
                        // Prepare, so we keep getting samples.
                        self.pcm
                            .device
                            .snd_pcm_prepare(&self.pcm.sound_device)
                            .unwrap();
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
