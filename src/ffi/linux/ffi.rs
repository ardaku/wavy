// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use self::alsa::{
    AlsaDevice, AlsaPlayer, AlsaRecorder, SndPcm, SndPcmAccess, SndPcmFormat,
    SndPcmHwParams, SndPcmMode, SndPcmState, SndPcmStream,
};
use asound::device_list::SoundDevice;
use fon::{
    chan::{Ch16, Channel},
    mono::Mono16,
    Sample,
    mono::Mono,
    stereo::Stereo16,
    surround::Surround16,
    Audio, Resampler, Stream,
};
use std::{
    convert::TryInto,
    future::Future,
    marker::PhantomData,
    os::raw::c_char,
    pin::Pin,
    task::{Context, Poll},
};

const CHANNEL_COUNT: &[u32] = &[8, 6, 4, 2, 1];

// Update with: `dl_api ffi/asound,so,2.muon src/linux/gen.rs`
mod alsa;
// ALSA bindings.
mod asound;

// Implementation Expectations:
pub(crate) use asound::device_list::{device_list, AudioDst, AudioSrc};

fn pcm_hw_params(
    device: &AlsaDevice,
    sound_device: &SndPcm,
    limit_buffer: bool,
    max_channels: u32,
) -> Option<(SndPcmHwParams, u32, u32)> {
    // unwrap: Allocating memory should not fail unless out of memory.
    let hw_params = device.snd_pcm_hw_params_malloc().unwrap();
    // Getting default settings should never fail.
    device.snd_pcm_hw_params_any(sound_device, &hw_params).ok()?;
    // Tell it not to resample
    device.snd_pcm_hw_params_set_rate_resample(sound_device, &hw_params, 0).ok()?;
    // Find something near 48_000 (this will prefer 48_000, but fallback to
    // 44_100, if that's what's supported.
    let mut sample_rate = 48_000;
    device
        .snd_pcm_hw_params_set_rate_near(
            sound_device,
            &hw_params,
            &mut sample_rate,
            None,
        ).ok()?;
    // Set access to RW noninterleaved.
    // Kernel should always support interleaved mode.
    device
        .snd_pcm_hw_params_set_access(
            sound_device,
            &hw_params,
            SndPcmAccess::RwInterleaved,
        ).ok()?;
    // S16LE should be supported by all popular speakers / microphones.
    device
        .snd_pcm_hw_params_set_format(
            sound_device,
            &hw_params,
            SndPcmFormat::S16Le,
        ).ok()?;
    // Get speaker configuration & number of channels.
    let mut ch_count = None;
    for count in CHANNEL_COUNT.iter().cloned() {
        if count > max_channels {
            continue;
        }
        if device
            .snd_pcm_hw_params_set_channels(sound_device, &hw_params, count)
            .is_ok()
        {
            ch_count = Some(count);
        }
    }
    let ch_count = ch_count?;
    // Period size must be a power of two
    // Currently only tries 1024
    let mut period_size = 1024;
    device
        .snd_pcm_hw_params_set_period_size_near(
            sound_device,
            &hw_params,
            &mut period_size,
            None,
        ).ok()?;
    // Set buffer size to about 3 times the period (setting latency).
    if limit_buffer {
        let mut buffer_size = 1024 * 3;
        // Some buffer size should always be available.
        device
            .snd_pcm_hw_params_set_buffer_size_near(
                sound_device,
                &hw_params,
                &mut buffer_size,
            ).ok()?;
    } else {
        // Apply the hardware parameters that just got set.
        // Should always be able to apply parameters that succeeded
        device.snd_pcm_hw_params(sound_device, &hw_params).ok()?;
        // Get rid of garbage.
        // Should always be able free data from the heap.
        device.snd_pcm_drop(&sound_device).ok()?;
    }
    // Re-Apply the hardware parameters that just got set.
    // Should always be able to apply parameters that succeeded
    device.snd_pcm_hw_params(sound_device, &hw_params).ok()?;

    Some((hw_params, sample_rate, ch_count))
}

// Speakers/Microphone Shared Code for ALSA.
pub(super) struct Pcm {
    device: AlsaDevice,
    sound_device: SndPcm,
    fd: smelling_salts::Device,
}

impl Pcm {
    /// Create a new async PCM.  If it fails return `None`.
    fn new(
        device_name: *const c_char,
        direction: SndPcmStream,
        max_channels: usize,
    ) -> Option<(Self, u32, u32)> {
        // Load shared alsa module.
        let device = AlsaDevice::new()?;
        // Create the ALSA PCM.
        let sound_device: SndPcm = device
            .snd_pcm_open(device_name, direction, SndPcmMode::Nonblock)
            .ok()?;
        // Configure Hardware Parameters
        let (mut hw_params, sample_rate, n_channels) = pcm_hw_params(
            &device,
            &sound_device,
            direction == SndPcmStream::Playback,
            max_channels.try_into().ok()?,
        )?;
        // Free Hardware Parameters
        device.snd_pcm_hw_params_free(&mut hw_params);
        // Get file descriptor
        let fd_count =
            device.snd_pcm_poll_descriptors_count(&sound_device).ok()?;
        let mut fd_list = Vec::with_capacity(fd_count.try_into().ok()?);
        device
            .snd_pcm_poll_descriptors(&sound_device, &mut fd_list)
            .ok()?;
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
            n_channels,
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
    buffer: Vec<[u8; 2]>,
    written: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample + Unpin> Future for &mut Speakers<S>
where
    Ch16: From<S::Chan>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.is_ready {
            if this.written >= 1024 {
                // Default to ready
                Poll::Ready(())
            } else {
                this.write();
                if this.written >= 1024 {
                    // Has read entire buffer, ready for more.
                    Poll::Ready(())
                } else {
                    // Buffer still has data, so still pending
                    // Next time this task wakes up, assume ready.
                    this.is_ready = true;
                    // Register waker, and then return not ready.
                    this.pcm.fd.register_waker(cx.waker());
                    Poll::Pending
                }
            }
        } else {
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
    pub(crate) fn connect(id: &crate::SpeakerId) -> Option<(Self, u32)> {
        // Load Player ALSA module
        let player = AlsaPlayer::new()?;
        // Create Playback PCM.
        let (pcm, sample_rate, nc) =
            Pcm::new(id.0.desc(), SndPcmStream::Playback, S::CHAN_COUNT)?;
        let buffer = vec![[0; 2]; nc as usize * 1024];
        let written = 0;
        let is_ready = true;
        let _phantom = PhantomData;

        Some((
            Self {
                player,
                pcm,
                is_ready,
                buffer,
                written,
                _phantom,
            },
            sample_rate,
        ))
    }

    pub(crate) fn play(&mut self, audio: &Audio<S>) {
        // Convert to speaker's native type.
        self.written = 0;
        match self.buffer.len() >> 10 {
            1 => {
                // Mono
                for (i, sample) in audio.iter().enumerate() {
                    let sample: Mono16 = sample.convert();
                    for (j, channel) in sample.channels().iter().enumerate() {
                        self.buffer[S::CHAN_COUNT * i + j] =
                            i16::from(*channel).to_le_bytes();
                    }
                }
            }
            2 => {
                // Stereo
                for (i, sample) in audio.iter().enumerate() {
                    let sample: Stereo16 = sample.convert();
                    for (j, channel) in sample.channels().iter().enumerate() {
                        self.buffer[S::CHAN_COUNT * i + j] =
                            i16::from(*channel).to_le_bytes();
                    }
                }
            }
            4 => {
                // Surround 4.0
                todo!()
            }
            6 => {
                // Surround 5.1
                for (i, sample) in audio.iter().enumerate() {
                    let sample: Surround16 = sample.convert();
                    for (j, channel) in sample.channels().iter().enumerate() {
                        self.buffer[S::CHAN_COUNT * i + j] =
                            i16::from(*channel).to_le_bytes();
                    }
                }
            }
            8 => {
                todo!()
            }
            _ => unreachable!(),
        }

        self.write();
    }

    #[allow(unsafe_code)]
    fn write(&mut self) {
        let ret = unsafe {
            self.player.snd_pcm_writei(
                &self.pcm.sound_device,
                self.buffer[self.written..].as_ptr().cast(),
                self.buffer[self.written..].len(),
            )
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
                                        self.buffer[..].as_ptr().cast(),
                                        self.buffer[..].len(),
                                    )
                                    .unwrap();
                            }
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
                                        self.buffer[..].as_ptr().cast(),
                                        self.buffer[..].len(),
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(len) => self.written += len as usize,
        }
    }
}

pub(crate) struct Microphone<C: Channel + Unpin> {
    recorder: AlsaRecorder,
    pcm: Pcm,
    is_ready: bool,
    stream: MicrophoneStream<C>,
}

impl<C: Channel + Unpin> Microphone<C> {
    pub(crate) fn new(id: &crate::MicrophoneId) -> Option<Self> {
        // Load Recorder ALSA module
        let recorder = AlsaRecorder::new()?;
        // Create Capture PCM.
        let (pcm, sample_rate, _one) =
            Pcm::new(id.0.desc(), SndPcmStream::Capture, 1)?;
        let is_ready = true;
        let stream = MicrophoneStream {
            buffer: Vec::with_capacity(1024),
            index: 0,
            resampler: Resampler::new(),
            sample_rate,
        };
        // Return successfully
        Some(Self {
            recorder,
            pcm,
            is_ready,
            stream,
        })
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.stream.sample_rate
    }

    pub(crate) fn record(&mut self) -> &mut MicrophoneStream<C> {
        // Reset everything
        self.stream.buffer.clear();
        self.stream.index = 0;
        // Record into temporary buffer.
        if let Err(error) = self
            .recorder
            .snd_pcm_readi(&self.pcm.sound_device, &mut self.stream.buffer)
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
                    eprintln!(
                        "Incorrect state (-EBADFD): Report Bug to \
                        https://github.com/libcala/wavy/issues/new"
                    );
                    unreachable!()
                }
                -32 => match state {
                    SndPcmState::Xrun => {
                        eprintln!("Microphone XRUN: Latency cause?");
                        self.pcm
                            .device
                            .snd_pcm_prepare(&self.pcm.sound_device)
                            .unwrap();
                    }
                    st => {
                        eprintln!(
                            "Incorrect state = {:?} (XRUN): Report Bug \
                            to https://github.com/libcala/wavy/issues/new",
                            st
                        );
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
        &mut self.stream
    }
}

impl<C: Channel + Unpin> Future for Microphone<C> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.is_ready {
            // Default to ready
            Poll::Ready(())
        } else {
            // Next time this task wakes up, assume ready.
            this.is_ready = true;
            // Register waker, and then return not ready.
            this.pcm.fd.register_waker(cx.waker());
            Poll::Pending
        }
    }
}

pub(crate) struct MicrophoneStream<C: Channel + Unpin> {
    // S16LE Audio Buffer
    buffer: Vec<[u8; 2]>,
    // Sample Rate of The Microphone (src)
    sample_rate: u32,
    // Index into buffer
    index: usize,
    // Stream's resampler
    resampler: Resampler<Mono<C>>,
}

impl<C> Stream<Mono<C>> for &mut MicrophoneStream<C>
where
    C: Channel + Unpin + From<Ch16>,
{
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn stream_sample(&mut self) -> Option<Mono<C>> {
        if self.index == self.buffer.len() {
            return None;
        }
        let sample: C =
            Ch16::from(i16::from_le_bytes(self.buffer[self.index])).into();
        self.index += 1;
        Some(Mono::new(sample))
    }

    fn resampler(&mut self) -> &mut Resampler<Mono<C>> {
        &mut self.resampler
    }
}
