use std::ffi::CString;
use std::task::{Waker, Poll};
use std::task::Context;
use std::pin::Pin;
use std::future::Future;
use std::convert::TryInto;
use std::iter::IntoIterator;
use std::borrow::Borrow;

use crate::gen::asound::{AlsaPlayer, AlsaRecorder, AlsaDevice, SndPcmStatus, SndPcmHwParams, SndPcmStream, SndPcm, SndPcmFormat, SndPcmAccess, SndAsyncHandler, SndPcmMode};
use crate::{AudioError, SampleRate, StereoS16Frame};

// A C Callback that gets called when an audio device's period boundary elapses.
// In ALSA this is implemented as a linux signal handler.
unsafe extern "C" fn elapse_period(handler: *mut std::os::raw::c_void) {
    let handler = SndAsyncHandler::from_raw(handler);
    let device = AlsaDevice::new().unwrap(); // Should never fail here
    let waker: *mut Option<Waker> = device.snd_async_handler_get_callback_private(&handler).cast();

    if let Some(waker) = (*waker).take() {
        waker.wake();
    }
}

fn pcm_hw_params(
    device: &AlsaDevice,
    sr: u32,
    sound_device: &SndPcm,
) -> Result<SndPcmHwParams, AudioError> {
    let hw_params = device.snd_pcm_hw_params_malloc().map_err(|_|
        AudioError::InternalError(
            "Cannot allocate hardware parameter structure!".to_string(),
        )
    )?;

    device.snd_pcm_hw_params_any(sound_device, &hw_params).map_err(|a|
        AudioError::InternalError(
            format!("Cannot initialize hardware parameter structure: {}!", a),
        )
    )?;
    // Enable resampling.
    device.snd_pcm_hw_params_set_rate_resample(sound_device, &hw_params, 1).map_err(|_|
        AudioError::InternalError(
            "Resampling setup failed for playback!".to_string(),
        )
    )?;
    // Set access to RW noninterleaved.
    device.snd_pcm_hw_params_set_access(sound_device, &hw_params,  SndPcmAccess::RwInterleaved).map_err(|_|
        AudioError::InternalError(
            "Cannot set access type!".to_string(),
        )
    )?;
    //
    device.snd_pcm_hw_params_set_format(sound_device, &hw_params, SndPcmFormat::S16Le).map_err(|_|
        AudioError::InternalError(
            "Cannot set sample format!".to_string(),
        )
    )?;
    // Set channels to stereo (2).
    device.snd_pcm_hw_params_set_channels(sound_device, &hw_params, 2).map_err(|_|
        AudioError::InternalError(
            "Cannot set channel count!".to_string(),
        )
    )?;
    // Set Sample rate.
    let mut actual_rate = sr;
    device.snd_pcm_hw_params_set_rate_near(
        sound_device,
        &hw_params,
        &mut actual_rate,
        None,
    ).map_err(|_|
        AudioError::InternalError(
            "Cannot set sample rate!".to_string(),
        )
    )?;
    if actual_rate != sr {
        return Err(AudioError::InternalError(format!(
            "Failed to set rate: {}, Got: {} instead!",
            sr, actual_rate
        )));
    }
    // Apply the hardware parameters that just got set.
    device.snd_pcm_hw_params(sound_device, &hw_params).map_err(|_|
        AudioError::InternalError(
            "Failed to set parameters!".to_string(),
        )
    )?;

    Ok(hw_params)
}

// Player/Recorder Shared Code for ALSA.
pub struct Pcm {
    device: AlsaDevice,
    sound_device: SndPcm,
    waker: Box<Option<Waker>>,
    handler: SndAsyncHandler,
    buffer_size: usize,
    period_size: usize,
}

impl Pcm {
    /// Create a new async PCM.
    fn new(direction: SndPcmStream, sr: u32) -> Result<Self, AudioError> {
        // Load shared alsa module.
        let device = AlsaDevice::new().ok_or_else(|| AudioError::InternalError(
            "Could not load AlsaDevice module in shared object!".to_string(),
        ))?;
        // Create a box for the waker, when it is created.
        let waker = Box::new(None);
        // FIXME: Currently only the default device is supported.
        let device_name = CString::new("default").unwrap();
        // Create the ALSA PCM.
        let sound_device: SndPcm = device.snd_pcm_open(
            &device_name, direction, SndPcmMode::Async
        ).map_err(|_| AudioError::NoDevice)?;
        // Configure Hardware Parameters
        let mut hw_params = pcm_hw_params(&device, sr, &sound_device)?;
        // Get the buffer size (in frames).
        let buffer_size = device.snd_pcm_hw_params_get_buffer_size(&hw_params)
.map_err(|_| AudioError::InternalError("Get Buffer Size".to_string()))?;
        // Get the period size (in frames).
        let mut d = 0;
        let period_size = device.snd_pcm_hw_params_get_period_size(
            &hw_params,
            Some(&mut d),
        ).map_err(|_| AudioError::InternalError("Get Period Size".to_string()))?;
        // Free Hardware Parameters
        device.snd_pcm_hw_params_free(&mut hw_params);
        // Make async signal handler
        let (waker, handler) = unsafe {
            let waker = Box::into_raw(waker);
            let handler = device.snd_async_add_pcm_handler(
                &sound_device,
                elapse_period as _,
                waker.cast(),
            );
            let waker = Box::from_raw(waker);

            (waker, handler)
        };

        Ok(Pcm {
            device, sound_device, waker, period_size, buffer_size, handler
        })
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        // Unregister async callback before boxed waker is dropped.
        self.device.snd_async_del_handler(&mut self.handler).unwrap();
        // Should never fail here
        self.device.snd_pcm_close(&mut self.sound_device).unwrap();
    }
}

impl Future for &mut Pcm {
    type Output = Result<usize, AudioError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let avail = self.device.snd_pcm_avail_update(&self.sound_device).map_err(|_|
            AudioError::InternalError("Couldn't get available".to_string())
        );
        let avail: usize = match avail {
            // Should never fail, negative is error.
            Ok(avail) => avail.try_into().unwrap(),
            Err(error) => return Poll::Ready(Err(error)),
        };
        if avail >= self.period_size {
            Poll::Ready(Ok(avail))
        } else {
            *self.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct Player {
    player: AlsaPlayer,
    pcm: Pcm,
    status: SndPcmStatus,
    buffer: Vec<u32>,
}

impl Player {
    pub fn new(sr: SampleRate) -> Result<Player, AudioError> {
        // Load Player ALSA module
        let player = AlsaPlayer::new().ok_or_else(|| AudioError::InternalError(
            "Could not load AlsaPlayer module in shared object!".to_string(),
        ))?;
        // Create Playback PCM.
        let pcm = Pcm::new(SndPcmStream::Playback, sr as u32)?;
        // Prepare PCM device
        player.snd_pcm_prepare(&pcm.sound_device).map_err(|_|
            AudioError::InternalError("Could not prepare!".to_string())
        )?;
        // FIXME: Do we need to get status with async?
        let status = pcm.device.snd_pcm_status_malloc().map_err(|_|
             AudioError::InternalError("Status alloc".to_string())
        )?;
        // Create buffer
        let buffer = Vec::new();

        Ok(Player {
            player,
            pcm,
            status,
            buffer,
        })
    }

    pub async fn play_last<T>(&mut self, iter: impl IntoIterator<Item=T>) -> Result<usize, AudioError>
        where T: Borrow<crate::StereoS16Frame>
    {
        let mut iter = iter.into_iter();

        // Wait for a number of frames to available.
        let nframes = (&mut self.pcm).await?;
        let mut avail: usize = nframes;

        // Add avail frames to the speaker's buffer.
        while avail >= self.pcm.period_size {
            // Clear the temporary buffer
            self.buffer.clear();
            // Write # of frames equal to the period size into the buffer.
            for _ in 0..self.pcm.period_size {
                let f = match iter.next() {
                    Some(f) => f.borrow().clone(),
                    None => StereoS16Frame::new(0, 0),
                };
                let [byte0, byte1] = f.left().to_le_bytes();
                let [byte2, byte3] = f.right().to_le_bytes();
                self.buffer.push(u32::from_le_bytes([byte0, byte1, byte2, byte3]));
            }
            // Copy the temporary buffer into the speaker hw buffer.
            let _ = self.player.snd_pcm_writei(
                &self.pcm.sound_device,
                &self.buffer,
            ).map_err(|_| println!("Buffer underrun"));
            // Update how many more frames need to be iterated.
            let avail2 = self.pcm.device.snd_pcm_avail_update(&self.pcm.sound_device).map_err(|_|
                AudioError::InternalError("Couldn't get available".to_string())
            );
            avail = match avail2 {
                // Should never fail, negative is error.
                Ok(avail3) => avail3.try_into().unwrap(),
                Err(error) => return Err(error),
            };
        }

        Ok(nframes)
    }
}

pub struct Recorder {
    recorder: AlsaRecorder,
    pcm: Pcm,
    status: SndPcmStatus,
    buffer: Vec<u32>,
    out: Vec<StereoS16Frame>,
}

impl Recorder {
    pub fn new(sr: SampleRate) -> Result<Recorder, AudioError> {
        // Load Recorder ALSA module
        let recorder = AlsaRecorder::new().ok_or_else(|| AudioError::InternalError(
            "Could not load AlsaRecorder module in shared object!".to_string(),
        ))?;
        // Create Capture PCM.
        let pcm = Pcm::new(SndPcmStream::Capture, sr as u32)?;
        // Start the PCM.
        pcm.device.snd_pcm_start(&pcm.sound_device).map_err(|_| 
            AudioError::InternalError("Could not start!".to_string())
        )?;
        // FIXME: Do we need to get status with async?
        let status = pcm.device.snd_pcm_status_malloc().map_err(|_| AudioError::InternalError("Status alloc".to_string()))?;
        // Create buffers
        let buffer = Vec::new();
        let out = Vec::new();

        Ok(Recorder {
            recorder,
            pcm,
            buffer,
            status,
            out,
        })
    }

    pub async fn record_last(&mut self) -> Result<&[StereoS16Frame], AudioError> {
        // Wait for a number of frames to available.
        let nframes = (&mut self.pcm).await?;
        let mut avail: usize = nframes;
        // Clear the output buffer
        self.out.clear();

        // Add avail frames to the speaker's buffer.
        while avail >= self.pcm.period_size {
            // Create temporary buffer.
            self.buffer = Vec::with_capacity(self.pcm.period_size);
            // Record into temporary buffer.
            let _ = self.recorder.snd_pcm_readi(
                &self.pcm.sound_device,
                &mut self.buffer,
            ).map_err(|_| println!("Buffer Overflow"));
            // Copy the temporary buffer into the output buffer
            for i in 0..self.pcm.period_size {
                let frame = self.buffer[i].to_le_bytes();
                let l = i16::from_le_bytes([frame[0], frame[1]]);
                let r = i16::from_le_bytes([frame[2], frame[3]]);

                self.out.push(StereoS16Frame::new(l, r));
            }
            // Update how many more frames need to be iterated.
            let avail2 = self.pcm.device.snd_pcm_avail_update(&self.pcm.sound_device).map_err(|_|
                AudioError::InternalError("Couldn't get available".to_string())
            );
            avail = match avail2 {
                // Should never fail, negative is error.
                Ok(avail3) => avail3.try_into().unwrap(),
                Err(error) => return Err(error),
            };
        }

        // assert_eq!(nframes, self.out.len());

        Ok(&self.out)
    }
}
