use std::ffi::CString;

use crate::gen::asound::{Player, Recorder, AudioDevice, SndPcmStatus, SndPcmHwParams, SndPcmStream, SndPcm, SndPcmFormat, SndPcmAccess};
use crate::{AudioError, SampleRate};

fn pcm_hw_params(
    device: &AudioDevice,
    sr: SampleRate,
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
    let mut actual_rate = sr as u32;
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
    if actual_rate != sr as u32 {
        return Err(AudioError::InternalError(format!(
            "Failed to set rate: {}, Got: {} instead!",
            sr as u32, actual_rate
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

pub struct Speaker {
    player: Player,
    device: AudioDevice,
    sound_device: SndPcm,
    buffer_size: u64,
    period_size: u64,
    status: SndPcmStatus,
    buffer: Vec<u32>,
}

impl Speaker {
    pub fn new(sr: SampleRate) -> Result<Speaker, AudioError> {
        let player = Player::new().ok_or_else(|| AudioError::InternalError(
            "Could not load Player module in shared object!".to_string(),
        ))?;
        let device = AudioDevice::new().ok_or_else(|| AudioError::InternalError(
            "Could not load AudioDevice module in shared object!".to_string(),
        ))?;

        // FIXME: Blocking IO => Use async
        let device_name = CString::new("default").unwrap();
        let sound_device: SndPcm = device.snd_pcm_open(
            &device_name, SndPcmStream::Playback, 0 /* blocking IO */
        ).map_err(|_| AudioError::NoDevice)?;

        let mut hw_params = pcm_hw_params(&device, sr, &sound_device)?;

        // Get the buffer size.
        let buffer_size = device.snd_pcm_hw_params_get_buffer_size(&hw_params)
.map_err(|_| AudioError::InternalError("Get Buffer Size".to_string()))?;
        let mut d = 0;
        let period_size = device.snd_pcm_hw_params_get_period_size(
            &hw_params,
            Some(&mut d),
        ).map_err(|_| AudioError::InternalError("Get Period Size".to_string()))?;

        device.snd_pcm_hw_params_free(&mut hw_params);

        player.snd_pcm_prepare(&sound_device).map_err(|_|
            AudioError::InternalError(
                "Could not prepare!".to_string(),
            )
        )?;

        let buffer_size = buffer_size as u64;
        let period_size = period_size as u64;

        let status = device.snd_pcm_status_malloc().map_err(|_| AudioError::InternalError("Status alloc".to_string()))?;
        let buffer = Vec::new();

        Ok(Speaker {
            player,
            device,
            sound_device,
            buffer_size,
            status,
            period_size,
            buffer,
        })
    }

    pub fn play(&mut self, generator: &mut dyn FnMut() -> (i16, i16)) {
        let _ = self.device.snd_pcm_status(
            &self.sound_device,
            &self.status,
        );
        let avail = self.device.snd_pcm_status_get_avail(&self.status);
        let left = self.buffer_size - avail as u64;

        let buffer_length = self.period_size * 4; // 16 bit (2 bytes) * Stereo (2 channels)

        let write = if left < buffer_length {
            buffer_length - left
        } else {
            0
        };

        self.buffer.clear();

        for _i in 0..write/2 {
            let (l, r) = generator();
            let [byte0, byte1] = l.to_le_bytes();
            let [byte2, byte3] = r.to_le_bytes();
            self.buffer.push(u32::from_le_bytes([byte0, byte1, byte2, byte3]));
        }

        let _ = self.player.snd_pcm_writei(
            &self.sound_device,
            &self.buffer,
        ).map_err(|_| println!("Buffer underrun"));
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        // Shouldn't fail
        self.device.snd_pcm_close(&mut self.sound_device).unwrap();
    }
}

pub struct Microphone {
    recorder: Recorder,
    device: AudioDevice,
    sound_device: SndPcm,
    status: SndPcmStatus,
    buffer: Vec<u32>,
}

impl Microphone {
    pub fn new(sr: SampleRate) -> Result<Microphone, AudioError> {
        let recorder = Recorder::new().ok_or_else(|| AudioError::InternalError(
            "Could not load Recorder module in shared object!".to_string(),
        ))?;
        let device = AudioDevice::new().ok_or_else(|| AudioError::InternalError(
            "Could not load AudioDevice module in shared object!".to_string(),
        ))?;

        // FIXME: Blocking IO => Use async
        let device_name = CString::new("default").unwrap();
        let sound_device: SndPcm = device.snd_pcm_open(
            &device_name, SndPcmStream::Capture, 0 /* blocking IO */
        ).map_err(|_| AudioError::NoDevice)?;

        let mut hw_params = pcm_hw_params(&device, sr, &sound_device)?;

        // FIXME? Get the buffer size.
        let _buffer_size = device.snd_pcm_hw_params_get_buffer_size(&hw_params).map_err(|_| AudioError::InternalError("Get Buffer Size".to_string()))?;

        let mut d = 0;
        // FIXME?
        let _period_size = device.snd_pcm_hw_params_get_period_size(
            &hw_params,
            Some(&mut d),
        ).map_err(|_| AudioError::InternalError("Get Period Size".to_string()))?;

        device.snd_pcm_hw_params_free(&mut hw_params);

        device.snd_pcm_start(&sound_device).map_err(|_| 
            AudioError::InternalError("Could not start!".to_string())
        )?;

        let status = device.snd_pcm_status_malloc().map_err(|_| AudioError::InternalError("Status alloc".to_string()))?;
        let buffer = Vec::new();

        Ok(Microphone {
            recorder,
            device,
            sound_device,
            buffer,
            status,
        })
    }

    pub fn record(&mut self, generator: &mut dyn FnMut(usize, i16, i16)) {
        let _ = self.device.snd_pcm_status(&self.sound_device, &self.status);
        let avail = self.device.snd_pcm_status_get_avail(&self.status);

        self.buffer = Vec::with_capacity(avail as usize);

        let _ = self.recorder.snd_pcm_readi(
            &self.sound_device,
            &mut self.buffer,
        ).map_err(|_| println!("Buffer Overflow"));

        for i in 0..avail as usize {
            let frame = self.buffer[i].to_le_bytes();
            let l = i16::from_le_bytes([frame[0], frame[1]]);
            let r = i16::from_le_bytes([frame[2], frame[3]]);

            generator(0, l, r);
        }
    }
}

impl Drop for Microphone {
    fn drop(&mut self) {
        // Shouldn't fail
        self.device.snd_pcm_close(&mut self.sound_device).unwrap();
    }
}
