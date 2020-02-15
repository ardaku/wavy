use std::ffi::CString;
use std::task::Poll;
use std::task::Context;
use std::pin::Pin;
use std::future::Future;
use std::convert::TryInto;
use std::iter::IntoIterator;
use std::borrow::Borrow;

// Update with: `dl_api ffi/asound,so,2.muon src/linux/gen.rs`
#[rustfmt::skip]
mod gen;

use self::gen::{AlsaPlayer, AlsaRecorder, AlsaDevice, SndPcmHwParams, SndPcmStream, SndPcm, SndPcmFormat, SndPcmAccess, SndPcmMode};
use crate::{AudioError, SampleRate, StereoS16Frame};

fn pcm_hw_params(
    device: &AlsaDevice,
    sr: u32,
    sound_device: &SndPcm,
    limit_buffer: bool,
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
    // Period size must be a power of two
    // Currently only tries 1024
    let mut period_size = 1024;
    device.snd_pcm_hw_params_set_period_size_near(
        sound_device,
        &hw_params,
        &mut period_size,
        None,
    ).unwrap();
    if period_size != 1024 {
        return Err(AudioError::InternalError(format!(
            "Wavy: Tried to set period size: {}, Got: {}!", 1024, period_size
        )));
    }
    // Set buffer size to about 3 times the period (setting latency).
    if limit_buffer {
        let mut buffer_size = period_size * 3;
        device.snd_pcm_hw_params_set_buffer_size_near(
            sound_device,
            &hw_params,
            &mut buffer_size,
        ).unwrap();
        if buffer_size != period_size * 3 {
            eprintln!(
                "Wavy: Tried to set buffer size: {}, Got: {}!",
                period_size * 3, buffer_size
            );
        }
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
    fd: smelling_salts::Device,
    period_size: usize,
}

impl Pcm {
    /// Create a new async PCM.
    fn new(direction: SndPcmStream, sr: u32) -> Result<Self, AudioError> {
        // Load shared alsa module.
        let device = AlsaDevice::new().ok_or_else(|| AudioError::InternalError(
            "Could not load AlsaDevice module in shared object!".to_string(),
        ))?;
        // FIXME: Currently only the default device is supported.
        let device_name = CString::new("default").unwrap();
        // Create the ALSA PCM.
        let sound_device: SndPcm = device.snd_pcm_open(
            &device_name, direction, SndPcmMode::Nonblock
        ).map_err(|_| AudioError::NoDevice)?;
        // Configure Hardware Parameters
        let mut hw_params = pcm_hw_params(&device, sr, &sound_device, direction == SndPcmStream::Playback)?;
        // Get the period size (in frames).
        let mut d = 0;
        let period_size = device.snd_pcm_hw_params_get_period_size(
            &hw_params,
            Some(&mut d),
        ).map_err(|_| AudioError::InternalError("Get Period Size".to_string()))?;
        // Free Hardware Parameters
        device.snd_pcm_hw_params_free(&mut hw_params);
        // Get file descriptor
        let fd_count = device.snd_pcm_poll_descriptors_count(&sound_device).unwrap();
        let mut fd_list = Vec::with_capacity(fd_count.try_into().unwrap());
        device.snd_pcm_poll_descriptors(&sound_device, &mut fd_list).unwrap();
        assert_eq!(fd_count, 1); // TODO: More?
        // Register file descriptor with OS's I/O Event Notifier
        let fd = smelling_salts::Device::new(
            fd_list[0].fd,
            match direction {
                SndPcmStream::Playback => smelling_salts::Direction::Write,
                SndPcmStream::Capture => smelling_salts::Direction::Read,
            },
        );

        Ok(Pcm {
            device, sound_device, period_size, fd
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

impl Future for &mut Pcm {
    type Output = Result<usize, AudioError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let avail = self.device.snd_pcm_avail_update(&self.sound_device).map_err(|_|
            AudioError::InternalError("Poll: Couldn't get available".to_string())
        );
        let avail: usize = match avail {
            // Should never fail, negative is error.
            Ok(avail) => avail.try_into().unwrap(),
            Err(error) => return Poll::Ready(Err(error)),
        };
        if avail >= self.period_size {
            Poll::Ready(Ok(avail))
        } else {
            self.fd.register_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct Player {
    player: AlsaPlayer,
    pcm: Pcm,
    buffer: Vec<StereoS16Frame>,
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
        // Create buffer
        let buffer = Vec::with_capacity(pcm.period_size);

        Ok(Player {
            player,
            pcm,
            buffer,
        })
    }

    #[allow(unsafe_code)]
    pub async fn play_last<T>(&mut self, iter: impl IntoIterator<Item=T>) -> Result<usize, AudioError>
        where T: Borrow<crate::StereoS16Frame>
    {
        let mut iter = iter.into_iter();

        // Wait for a number of frames to available.
        let nframes = (&mut self.pcm).await?;

        // Add avail frames to the speaker's buffer. //

        // Clear the temporary buffer
        self.buffer.clear();
        // Write # of frames equal to the period size into the buffer.
        for _ in 0..self.pcm.period_size {
            let f = match iter.next() {
                Some(f) => f.borrow().clone(),
                None => StereoS16Frame::new(0, 0),
            };
            self.buffer.push(f);
        }
        // Copy the temporary buffer into the speaker hw buffer.
        let len = self.player.snd_pcm_writei(
            &self.pcm.sound_device,
            unsafe { std::mem::transmute(self.buffer.as_slice()) },
        ).map_err(|_| println!("Buffer underrun")).unwrap() as usize;
        assert_eq!(len, self.buffer.len());
        // Update how many more frames need to be iterated.
        self.pcm.device.snd_pcm_avail_update(&self.pcm.sound_device).map_err(|_|
            AudioError::InternalError("Play: Couldn't get available".to_string())
        )?;

        Ok(nframes)
    }
}

pub struct Recorder {
    recorder: AlsaRecorder,
    pcm: Pcm,
    buffer: Vec<StereoS16Frame>,
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
        // Create buffers (TODO: Shouldn't be necessary to have 2 buffers)
        let buffer = Vec::with_capacity(pcm.period_size);
        // Return successfully
        Ok(Recorder {
            recorder,
            pcm,
            buffer,
        })
    }

    pub async fn record_last(&mut self) -> Result<&[StereoS16Frame], AudioError> {
        /*// Wait for a number of frames to available.
        let _nframes = (&mut self.pcm).await?;
        // Clear the output buffer
        self.buffer.clear();

        // Add avail frames to the speaker's buffer. //

        // Record into temporary buffer.
        self.recorder.snd_pcm_readi(
            &self.pcm.sound_device,
            unsafe { std::mem::transmute(&mut self.buffer) },
        ).map_err(|_| println!("Buffer Overflow")).unwrap();
        // Update how many more frames need to be iterated.
        self.pcm.device.snd_pcm_avail_update(&self.pcm.sound_device).map_err(|_|
            AudioError::InternalError("Record: Couldn't get available".to_string())
        )?;*/

        (&mut *self).await;
        Ok(&self.buffer)
    }
}

impl Future for &mut Recorder {
    type Output = ();

    #[allow(unsafe_code)]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = Pin::into_inner(self);
        // Clear the output buffer (Keeps capacity of 1 period the same)
        this.buffer.clear();
        // Record into temporary buffer.
        if let Err(error) = this.recorder.snd_pcm_readi(
            &this.pcm.sound_device,
            unsafe { std::mem::transmute(&mut this.buffer) },
        )
        {
            match error {
                -77 => panic!("Incorrect state (-EBADFD): FIXME"),
                -32 => panic!("Buffer Overrun (Underflow): FIXME"),
                -86 => panic!("Stream got suspended, must be recovered (-ESTRPIPE): FIXME"),
                a => {
                    println!("error is: {}", a);
                    unreachable!()
                },
            }
        }
        // Return
        match this.buffer.len() {
            // No Data was read, go to sleep
            0 => {
                this.pcm.fd.register_waker(cx.waker().clone());
                Poll::Pending
            }
            _ => {
                Poll::Ready(())
            }
/*            // A period of data was read
            x if x == this.pcm.period_size => {
                Poll::Ready(())
            }
            // Incomplete Read
            s => panic!("Incomplete Read of size {}!  Shouldn't Happen", s)*/
        }
    }
}
