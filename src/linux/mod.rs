use crate::frame::Frame;
use std::convert::TryInto;
use std::ffi::CString;
use std::marker::PhantomData;
use std::task::Context;
use std::task::Poll;

// Update with: `dl_api ffi/asound,so,2.muon src/linux/gen.rs`
#[rustfmt::skip]
mod gen;

use self::gen::{
    AlsaDevice, AlsaPlayer, AlsaRecorder, SndPcm, SndPcmAccess, SndPcmFormat,
    SndPcmHwParams, SndPcmMode, SndPcmState, SndPcmStream,
};

fn pcm_hw_params(
    device: &AlsaDevice,
    sr: u32,
    sound_device: &SndPcm,
    limit_buffer: bool,
) -> Option<SndPcmHwParams> {
    // unwrap: Allocating memory should not fail unless out of memory.
    let hw_params = device.snd_pcm_hw_params_malloc().unwrap();
    // unwrap: Getting default settings should never fail.
    device
        .snd_pcm_hw_params_any(sound_device, &hw_params)
        .unwrap();
    // Enable resampling.
    // unwrap: FIXME: Fallback SIMD resampler if this were to fail.
    device
        .snd_pcm_hw_params_set_rate_resample(sound_device, &hw_params, 1)
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
    // unwrap: FIXME: Fallback SIMD resampler if this were to fail.
    device
        .snd_pcm_hw_params_set_format(
            sound_device,
            &hw_params,
            SndPcmFormat::S16Le,
        )
        .unwrap();
    // Set channels to stereo (2).
    // unwrap: FIXME: Fallback SIMD resampler if this were to fail.
    device
        .snd_pcm_hw_params_set_channels(sound_device, &hw_params, 2)
        .unwrap();
    // Set Sample rate.
    // unwrap: FIXME: Fallback SIMD resampler if this were to fail.
    let mut actual_rate = sr;
    device
        .snd_pcm_hw_params_set_rate_near(
            sound_device,
            &hw_params,
            &mut actual_rate,
            None,
        )
        .unwrap();
    if actual_rate != sr {
        panic!("Failed to set rate: {}, Got: {} instead!", sr, actual_rate);
    }
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

    Some(hw_params)
}

// Player/Recorder Shared Code for ALSA.
pub struct Pcm {
    device: AlsaDevice,
    sound_device: SndPcm,
    fd: smelling_salts::Device,
}

impl Pcm {
    /// Create a new async PCM.
    fn new(direction: SndPcmStream, sr: u32) -> Option<Self> {
        // Load shared alsa module.
        let device = AlsaDevice::new()?;
        // FIXME: Currently only the default device is supported.
        let device_name = CString::new("default").unwrap();
        // Create the ALSA PCM.
        let sound_device: SndPcm = device
            .snd_pcm_open(&device_name, direction, SndPcmMode::Nonblock)
            .ok()?;
        // Configure Hardware Parameters
        let mut hw_params = pcm_hw_params(
            &device,
            sr,
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

        Some(Pcm {
            device,
            sound_device,
            fd,
        })
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

pub(crate) struct Player<F: Frame> {
    player: AlsaPlayer,
    pcm: Pcm,
    is_ready: bool,
    _phantom: PhantomData<F>,
}

impl<F: Frame> Player<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        // Load Player ALSA module
        let player = AlsaPlayer::new()?;
        // Create Playback PCM.
        let pcm = Pcm::new(SndPcmStream::Playback, sr)?;
        let is_ready = true;
        let _phantom = PhantomData;

        Some(Player {
            player,
            pcm,
            is_ready,
            _phantom,
        })
    }

    pub(crate) fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        if self.is_ready {
            // Default to ready
            Poll::Ready(())
        } else {
            // Next time this task wakes up, assume ready.
            self.is_ready = true;
            // Register waker, and then return not ready.
            self.pcm.fd.register_waker(cx.waker().clone());
            Poll::Pending
        }
    }

    pub(crate) fn play_last(&mut self, audio: &[F]) -> usize {
        let silence = [F::default(); 1024];
        let audio = if audio.is_empty() {
            &silence
        } else {
            audio
        };

        // Record into temporary buffer.
        match self.player.snd_pcm_writei(&self.pcm.sound_device, audio) {
            Err(error) => {
                let state = self.pcm.device.snd_pcm_state(&self.pcm.sound_device);
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
                            self.player
                                .snd_pcm_writei(
                                    &self.pcm.sound_device,
                                    &silence,
                                )
                                .unwrap();
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
                            self.player
                                .snd_pcm_writei(
                                    &self.pcm.sound_device,
                                    &silence,
                                )
                                .unwrap();
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
    is_ready: bool,
    _phantom: PhantomData<F>,
}

impl<F: Frame> Recorder<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        // Load Recorder ALSA module
        let recorder = AlsaRecorder::new()?;
        // Create Capture PCM.
        let pcm = Pcm::new(SndPcmStream::Capture, sr)?;
        // pcm.device.snd_pcm_start(&pcm.sound_device).unwrap();
        let is_ready = true;
        let _phantom = PhantomData;
        // Return successfully
        Some(Recorder {
            recorder,
            pcm,
            is_ready,
            _phantom,
        })
    }

    pub(crate) fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        if self.is_ready {
            // Default to ready
            Poll::Ready(())
        } else {
            // Next time this task wakes up, assume ready.
            self.is_ready = true;
            // Register waker, and then return not ready.
            self.pcm.fd.register_waker(cx.waker().clone());
            Poll::Pending
        }
    }

    pub(crate) fn record_last(&mut self, audio: &mut Vec<F>) {
            let state = self.pcm.device.snd_pcm_state(&self.pcm.sound_device);

        // Record into temporary buffer.
        if let Err(error) = self.recorder.snd_pcm_readi(&self.pcm.sound_device, audio) {
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
                    if self.pcm
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
