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
    AlsaDevice, AlsaPlayer, SndPcm, SndPcmAccess, SndPcmFormat,
    SndPcmHwParams, SndPcmMode, SndPcmState, SndPcmStream,
};
use asound::device_list::SoundDevice;
use fon::{chan::{Ch32, Channel}, Frame, Resampler, Stream, Sink, surround::Surround32};
use std::{
    convert::TryInto,
    future::Future,
    os::raw::c_char,
    pin::Pin,
    task::{Context, Poll},
    marker::PhantomData,
    mem::MaybeUninit,
};

// Update with: `dl_api ffi/asound,so,2.muon src/linux/gen.rs`
mod alsa;
// ALSA bindings.
mod asound;

// Implementation Expectations:
pub(crate) use asound::device_list::{device_list, AudioDst, AudioSrc};

#[allow(unsafe_code)]
fn pcm_hw_params(
    sound_device: &SndPcm,
    channels: u8,
) -> Option<(f64, u16, u8)> {
    unsafe {
    // unwrap: Allocating memory should not fail unless out of memory.
    let mut hw_params = MaybeUninit::uninit();
    asound::pcm::hw_params_malloc(hw_params.as_mut_ptr()).unwrap();
    let hw_params = hw_params.assume_init();
    // Getting default settings should never fail.
    asound::pcm::hw_params_any(sound_device.0, hw_params).ok()?;
    // Set the number of channels.
    asound::pcm::hw_set_channels(sound_device.0, hw_params, channels.into()).expect(&format!("Failed to use {} channels on speaker", channels));
    // Set Hz near library target Hz.
    asound::pcm::hw_params_set_rate_near(
        sound_device.0,
        hw_params,
        &mut crate::consts::SAMPLE_RATE.into(),
        &mut 0,
    ).ok()?;
    // Fon uses interleaved audio, so set device as interleaved.
    // Kernel should always support RW interleaved mode.
    asound::pcm::hw_params_set_access(
        sound_device.0,
        hw_params,
        SndPcmAccess::RwInterleaved,
    ).ok()?;
    // Request 32-bit Float.
    asound::pcm::hw_params_set_format(
        sound_device.0,
        hw_params,
        if cfg!(target_endian = "little") {
            SndPcmFormat::FloatLe
        } else if cfg!(target_endian = "big") {
            SndPcmFormat::FloatBe
        } else {
            unreachable!()
        },
    ).ok()?;
    // Set period near library target period.
    let mut period_size = crate::consts::PERIOD.into();
    asound::pcm::hw_params_set_period_size_near(
        sound_device.0,
        hw_params,
        &mut period_size,
        &mut 0,
    ).ok()?;
    let period_size: u16 = period_size.try_into().ok()?;
    // Set buffer size near period * 2.
    let mut buffer_size = (period_size * 2).into();
    // Some buffer size should always be available.
    asound::pcm::hw_params_set_buffer_size_near(
        sound_device.0,
        hw_params,
        &mut buffer_size,
    ).ok()?;
    // Should always be able to apply parameters that succeeded
    asound::pcm::hw_params(sound_device.0, hw_params).ok()?;
    // Now that a configuration has been chosen, we can retreive the actual
    // exact sample rate.
    let sample_rate = asound::pcm::get_rate(hw_params)?;
    // Retreive the number of channels.
    let channels = asound::pcm::get_channels(hw_params);
    // Free Hardware Parameters
    asound::pcm::hw_params_free(hw_params);

    Some((sample_rate, period_size, channels))
    }
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
    ) -> Option<Self> {
        // Load shared alsa module.
        let device = AlsaDevice::new()?;
        // Create the ALSA PCM.
        let sound_device: SndPcm = device
            .snd_pcm_open(device_name, direction, SndPcmMode::Nonblock)
            .ok()?;
        // FIXME
        /*// If direction is Recording, then make sure the audio buffer is empty.
        if direction == SndPcmStream::Capture {
            device.snd_pcm_drop(&sound_device).ok()?;
            device.snd_pcm_prepare(&sound_device).ok()?;
        }*/
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

        Some(
            Pcm {
                device,
                sound_device,
                fd,
            },
        )
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

/// ALSA Speakers connection.
pub(crate) struct Speakers {
    /// FIXME: Remove in favor of new DL_API
    player: AlsaPlayer,
    /// ALSA PCM type for both speakers and microphones.
    pcm: Pcm,
    /// How many audio frames have been written to the speaker buffer.
    written: usize,
    /// Raw buffer of audio yet to be played.
    buffer: Vec<Ch32>,
    /// Resampler context for speakers sink.
    resampler: ([Ch32; 6], f64),
    /// The number of frames in the buffer.
    period: u16,
    /// Number of available channels
    pub(crate) channels: u8,
    /// The sample rate of the speakers.
    pub(crate) sample_rate: Option<f64>,
}

impl Speakers {
    pub(crate) fn connect(id: &crate::SpeakersId) -> Option<Self> {
        // Load Player ALSA module
        let player = AlsaPlayer::new()?;
        // Create Playback PCM.
        let pcm = Pcm::new(id.0.desc(), SndPcmStream::Playback)?;
        Some(
            Self {
                player,
                pcm,
                written: 0,
                buffer: Vec::new(),
                sample_rate: None,
                channels: 0,
                resampler: ([Ch32::MID; 6], 0.0),
                period: 0,
            }
        )
    }

    #[allow(unsafe_code)]
    pub(crate) fn play<F: Frame<Chan = Ch32>>(&mut self) -> SpeakersSink<'_, F> {
        if F::CHAN_COUNT != self.channels.into() {
            if !matches!(F::CHAN_COUNT, 1 | 2 | 6) {
                panic!("Unknown speaker configuration")
            }
            // Configure Hardware Parameters
            let (sample_rate, period, channels) = pcm_hw_params(
                &self.pcm.sound_device,
                F::CHAN_COUNT as u8,
            ).expect("Failed to adjust hardware parameters.  This is a Wavy bug.");
            // Resize the buffer
            self.buffer.resize(period as usize * channels as usize, Ch32::MID);
            // Set the sample rate.
            self.sample_rate = Some(sample_rate);
            // 
            self.period = period;
            // 
            self.channels = channels;
        }
        let resampler = Resampler::<F>::new(
            Surround32::from_channels(&self.resampler.0[..]).convert(),
            self.resampler.1
        );

        // Reset the Speakers index of written frames.  When the number of
        // written frames reaches the period, then the future will return Ready.
        self.written = 0;
        // Create a sink that borrows this speaker's buffer mutably.
        SpeakersSink(self, resampler, PhantomData)
    }
}

impl Future for &mut Speakers {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to speakers.
        let this = self.get_mut();

        // Check if speakers have been used yet, if not return Ready.
        if this.channels == 0 {
            return Poll::Ready(());
        }

        // Attempt to write remaining internal speaker buffer to the speakers.
        let result = unsafe {
            this.player.snd_pcm_writei(
                &this.pcm.sound_device,
                this.buffer[this.written..].as_ptr().cast(),
                this.buffer[this.written..].len(),
            )
        };

        // Check if it succeeds, then return Ready.
        match result {
            Err(error) => {
                match error {
                    // Edge-triggered epoll should only go into pending mode if
                    // read/write call results in EAGAIN (according to epoll man
                    // page)
                    -11 => { /* Pending */ }
                    -32 => match this.pcm.device.snd_pcm_state(&this.pcm.sound_device) {
                        SndPcmState::Xrun => {
                            // Player samples are not generated fast enough
                            this.pcm
                                .device
                                .snd_pcm_prepare(&this.pcm.sound_device)
                                .unwrap();
                            unsafe {
                                this.player
                                    .snd_pcm_writei(
                                        &this.pcm.sound_device,
                                        this.buffer[this.written..].as_ptr().cast(),
                                        this.buffer[this.written..].len(),
                                    )
                                    .unwrap();
                            }
                        }
                        st => {
                            eprintln!(
                                "Incorrect state = {:?} (XRUN): Report Bug to \
                                 https://github.com/libcala/wavy/issues/new",
                                st
                            );
                            unreachable!()
                        }
                    },
                    -77 => {
                        eprintln!(
                            "Incorrect state (-EBADFD): Report Bug to \
                             https://github.com/libcala/wavy/issues/new");
                        unreachable!()
                    }
                    -86 => {
                        eprintln!("Stream got suspended, trying to recover… \
                            (-ESTRPIPE)");
                        if this
                            .pcm
                            .device
                            .snd_pcm_resume(&this.pcm.sound_device)
                            .is_ok()
                        {
                            // Prepare, so we keep getting samples.
                            this.pcm
                                .device
                                .snd_pcm_prepare(&this.pcm.sound_device)
                                .unwrap();
                            unsafe {
                                this.player
                                    .snd_pcm_writei(
                                        &this.pcm.sound_device,
                                        this.buffer[this.written..].as_ptr().cast(),
                                        this.buffer[this.written..].len(),
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(len) => {
                this.written += len as usize;
                if this.written >= this.period.into() {
                    return Poll::Ready(());
                }
            }
        }

        // Register waker, and then return not ready.
        this.pcm.fd.register_waker(cx.waker());
        Poll::Pending
    }
}

pub(crate) struct SpeakersSink<'a, F: Frame<Chan = Ch32>>(&'a mut Speakers, Resampler<F>, PhantomData<F>);

impl<F: Frame<Chan = Ch32>> Sink<F> for SpeakersSink<'_, F> {
    fn sample_rate(&self) -> f64 {
        self.0.sample_rate.unwrap()
    }

    fn resampler(&mut self) -> &mut Resampler<F> {
        &mut self.1
    }

    #[allow(unsafe_code)]
    fn buffer(&mut self) -> &mut [F] {
        let data = self.0.buffer.as_mut_ptr().cast();
        let count = self.0.period.into();
        unsafe {
            std::slice::from_raw_parts_mut(data, count)
        }
    }
}

impl<F: Frame<Chan = Ch32>> Drop for SpeakersSink<'_, F> {
    fn drop(&mut self) {
        // Store 5.1 surround sample to resampler.
        let frame: Surround32 = self.1.frame().convert();
        self.0.resampler.0 = [
            frame.channels()[0],
            frame.channels()[1],
            frame.channels()[2],
            frame.channels()[3],
            frame.channels()[4],
            frame.channels()[5],
        ];
        // Store partial index from resampler.
        self.0.resampler.1 = self.1.index() % 1.0;
    }
}

pub(crate) struct Microphone {
    // PCM I/O Handle
    pcm: Pcm,
    // Interleaved Audio Buffer.
    buffer: Vec<Ch32>,
    // The period of the microphone.
    period: u16,
    // Number of channels on the Microphone.
    pub(crate) channels: u8,
    // Sample Rate of The Microphone (src)
    pub(crate) sample_rate: Option<f64>,
}

impl Microphone {
    pub(crate) fn new(id: &crate::MicrophoneId) -> Option<Self> {
        // Create Capture PCM.
        let pcm = Pcm::new(id.0.desc(), SndPcmStream::Capture)?;
        // Return successfully
        Some(Self {
            pcm,
            buffer: Vec::new(),
            period: 0,
            channels: 0,
            sample_rate: None,
        })
    }

    pub(crate) fn record<F: Frame<Chan = Ch32>>(&mut self) -> MicrophoneStream<'_, F> {
        // Stream from microphone's buffer.
        MicrophoneStream(self, 0, PhantomData)
    }
}

impl Future for Microphone {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Get mutable reference to microphone.
        let this = self.get_mut();

        // Attempt to overwrite the internal microphone buffer.
        let result = unsafe {
            asound::pcm::readi(this.pcm.sound_device.0,
                this.buffer.as_mut_slice().as_mut_ptr(), this.period)
        };

        // Check if it succeeds, then return Ready.
        if let Err(error) = result {
            match error {
                // Edge-triggered epoll should only go into pending mode if
                // read/write call results in EAGAIN (according to epoll man
                // page)
                -11 => { /* Pending */ }
                -77 => {
                    eprintln!(
                        "Incorrect state (-EBADFD): Report Bug to \
                        https://github.com/libcala/wavy/issues/new"
                    );
                    unreachable!()
                }
                -32 => match this.pcm.device.snd_pcm_state(&this.pcm.sound_device) {
                    SndPcmState::Xrun => {
                        eprintln!("Microphone XRUN: Latency cause?");
                        this.pcm
                            .device
                            .snd_pcm_prepare(&this.pcm.sound_device)
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
                    if this
                        .pcm
                        .device
                        .snd_pcm_resume(&this.pcm.sound_device)
                        .is_ok()
                    {
                        // Prepare, so we keep getting samples.
                        this.pcm
                            .device
                            .snd_pcm_prepare(&this.pcm.sound_device)
                            .unwrap();
                    }
                }
                _ => unreachable!(),
            }
            // Register waker
            this.pcm.fd.register_waker(cx.waker());
            // Not ready
            Poll::Pending
        } else {
            // Ready, audio buffer has been filled!
            Poll::Ready(())
        }
    }
}

pub(crate) struct MicrophoneStream<'a, F: Frame<Chan = Ch32>>(
    &'a mut Microphone,
    usize,
    PhantomData<F>,
);

impl<F: Frame<Chan = Ch32>> Iterator for MicrophoneStream<'_, F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0.period.into() {
            return None;
        }
        Some(F::from_channels(&self.0.buffer[self.1 * self.0.channels as usize..]))
    }
}

impl<F: Frame<Chan = Ch32>> Stream<F> for MicrophoneStream<'_, F> {
    fn sample_rate(&self) -> Option<f64> {
        self.0.sample_rate
    }

    fn len(&self) -> Option<usize> {
        Some(self.0.period.into())
    }
}
