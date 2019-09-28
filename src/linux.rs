#![allow(non_camel_case_types)]

use crate::*;

use libc;
use libc::{c_char, c_int, c_long, c_uint, c_ulong, size_t};

enum snd_pcm_t {}
enum snd_pcm_hw_params_t {}
enum snd_pcm_status_t {}

type snd_pcm_stream_t = c_uint;
type snd_pcm_access_t = c_uint;
type snd_pcm_format_t = c_int;
type snd_pcm_sframes_t = c_long;

#[link(name = "asound")]
extern "C" {
    fn snd_pcm_open(
        pcm: *mut *mut snd_pcm_t,
        name: *const c_char,
        stream: snd_pcm_stream_t,
        mode: c_int,
    ) -> c_int;

    fn snd_pcm_hw_params_malloc(ptr: *mut *mut snd_pcm_hw_params_t) -> c_int;

    fn snd_pcm_hw_params_any(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> c_int;

    fn snd_pcm_hw_params_set_rate_resample(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        a: c_uint,
    ) -> c_int;

    fn snd_pcm_hw_params_set_access(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        _access: snd_pcm_access_t,
    ) -> c_int;

    fn snd_pcm_hw_params_set_format(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: snd_pcm_format_t,
    ) -> c_int;

    fn snd_pcm_hw_params_set_channels(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: c_uint,
    ) -> c_int;

    fn snd_pcm_hw_params_set_rate_near(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: *mut c_uint,
        dir: *mut c_int,
    ) -> c_int;

    fn snd_pcm_hw_params(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> c_int;

    fn snd_pcm_hw_params_get_buffer_size(
        params: *const snd_pcm_hw_params_t,
        val: *mut c_ulong,
    ) -> c_int;

    fn snd_pcm_hw_params_get_period_size(
        params: *const snd_pcm_hw_params_t,
        frames: *mut c_ulong,
        dir: *mut c_int,
    ) -> c_int;

    fn snd_pcm_hw_params_free(obj: *mut snd_pcm_hw_params_t);

    fn snd_pcm_prepare(pcm: *mut snd_pcm_t) -> c_int;

    fn snd_pcm_writei(pcm: *mut snd_pcm_t, buffers: *mut i16, size: c_ulong) -> snd_pcm_sframes_t;

    fn snd_pcm_readi(pcm: *mut snd_pcm_t, buffer: *mut i16, size: c_ulong) -> snd_pcm_sframes_t;

    fn snd_pcm_status_sizeof() -> size_t;

    fn snd_pcm_status(pcm: *mut snd_pcm_t, status: *mut snd_pcm_status_t) -> c_int;

    fn snd_pcm_status_get_avail(obj: *const snd_pcm_status_t) -> c_ulong;

    fn snd_pcm_close(pcm: *mut snd_pcm_t) -> c_int;

    fn snd_pcm_start(pcm: *mut snd_pcm_t) -> c_int;
}

// Connect to a speaker or microphone.
fn pcm_open(microphone: bool, name: &[u8]) -> Result<*mut snd_pcm_t, AudioError> {
    let mut sound_device: *mut snd_pcm_t = unsafe { std::mem::uninitialized() };
    if unsafe {
        // plughw:0,0 ?
        snd_pcm_open(
            &mut sound_device,
            name.as_ptr() as *const _,
            if microphone {
                1 /* capture */
            } else {
                0 /*playback*/
            },
            0,
        )
    } < 0
    {
        return Err(AudioError::NoDevice);
    }
    Ok(sound_device)
}

fn pcm_hw_params(
    sr: SampleRate,
    sound_device: *mut snd_pcm_t,
) -> Result<*mut snd_pcm_hw_params_t, AudioError> {
    let mut hw_params: *mut snd_pcm_hw_params_t = unsafe { std::mem::uninitialized() };
    if unsafe { snd_pcm_hw_params_malloc(&mut hw_params) } < 0 {
        return Err(AudioError::InternalError(
            "Cannot allocate hardware parameter structure!".to_string(),
        ));
    }
    if unsafe { snd_pcm_hw_params_any(sound_device, hw_params) } < 0 {
        return Err(AudioError::InternalError(
            "Cannot initialize hardware parameter structure!".to_string(),
        ));
    }
    // Enable resampling.
    if unsafe { snd_pcm_hw_params_set_rate_resample(sound_device, hw_params, 1) } < 0 {
        return Err(AudioError::InternalError(
            "Resampling setup failed for playback!".to_string(),
        ));
    }
    // Set access to RW noninterleaved.
    if unsafe {
        snd_pcm_hw_params_set_access(sound_device, hw_params, 3 /*RW_INTERLEAVED*/)
    } < 0
    {
        return Err(AudioError::InternalError(
            "Cannot set access type!".to_string(),
        ));
    }
    //
    if unsafe {
        snd_pcm_hw_params_set_format(sound_device, hw_params, 2 /*S16_LE*/)
    } < 0
    {
        return Err(AudioError::InternalError(
            "Cannot set sample format!".to_string(),
        ));
    }
    // Set channels to stereo (2).
    if unsafe { snd_pcm_hw_params_set_channels(sound_device, hw_params, 2) } < 0 {
        return Err(AudioError::InternalError(
            "Cannot set channel count!".to_string(),
        ));
    }
    // Set Sample rate.
    let mut actual_rate = sr as u32;
    if unsafe {
        snd_pcm_hw_params_set_rate_near(
            sound_device,
            hw_params,
            &mut actual_rate,
            std::ptr::null_mut(),
        )
    } < 0
    {
        return Err(AudioError::InternalError(
            "Cannot set sample rate!".to_string(),
        ));
    }
    if actual_rate != sr as u32 {
        return Err(AudioError::InternalError(format!(
            "Failed to set rate: {}, Got: {} instead!",
            sr as u32, actual_rate
        )));
    }
    // Apply the hardware parameters that just got set.
    if unsafe { snd_pcm_hw_params(sound_device, hw_params) } < 0 {
        return Err(AudioError::InternalError(
            "Failed to set parameters!".to_string(),
        ));
    }

    Ok(hw_params)
}

pub struct Speaker {
    sound_device: *mut snd_pcm_t,
    buffer_size: u64,
    period_size: u64,
    status: Vec<u8>,
    buffer: Vec<i16>,
}

impl Speaker {
    pub fn new(sr: SampleRate) -> Result<Speaker, AudioError> {
        let sound_device: *mut snd_pcm_t = pcm_open(false, b"default\0")?;
        let hw_params = pcm_hw_params(sr, sound_device)?;

        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        //        dbg!(buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(hw_params, &mut period_size, &mut d);
        }
        //        dbg!(period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe { snd_pcm_prepare(sound_device) } < 0 {
            return Err(AudioError::InternalError("Could not prepare!".to_string()));
        }

        let buffer_size = buffer_size as u64;
        let period_size = period_size as u64;

        let status = vec![0; unsafe { snd_pcm_status_sizeof() }];
        let buffer = Vec::new();

        Ok(Speaker {
            sound_device,
            buffer_size,
            status,
            period_size,
            buffer,
        })
    }

    pub fn play(&mut self, generator: &mut dyn FnMut() -> (i16, i16)) {
        let _ = unsafe { snd_pcm_status(self.sound_device, self.status.as_mut_ptr() as *mut _) };
        let avail = unsafe { snd_pcm_status_get_avail(self.status.as_ptr() as *const _) };
        let left = self.buffer_size - avail;

        let buffer_length = self.period_size * 4; // 16 bit (2 bytes) * Stereo (2 channels)

        let write = if left < buffer_length {
            buffer_length - left
        } else {
            0
        };

        self.buffer.clear();

        for _i in 0..write {
            let (l, r) = generator();
            self.buffer.push(l);
            self.buffer.push(r);
        }

        if unsafe { snd_pcm_writei(self.sound_device, self.buffer.as_mut_ptr(), write as u64) } < 0
        {
            println!("Buffer underrun");
        }
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        unsafe {
            snd_pcm_close(self.sound_device);
        }
    }
}

pub struct Microphone {
    sound_device: *mut snd_pcm_t,
    status: Vec<u8>,
    buffer: Vec<i16>,
}

impl Microphone {
    pub fn new(sr: SampleRate) -> Result<Microphone, AudioError> {
        let sound_device: *mut snd_pcm_t = pcm_open(true, b"default\0")?;
        let hw_params = pcm_hw_params(sr, sound_device)?;

        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        //        dbg!(buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(hw_params, &mut period_size, &mut d);
        }
        //        dbg!(period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe { snd_pcm_start(sound_device) } < 0 {
            return Err(AudioError::InternalError("Could not start!".to_string()));
        }

        let status = vec![0; unsafe { snd_pcm_status_sizeof() }];
        let buffer = Vec::new();

        Ok(Microphone {
            sound_device,
            buffer,
            status,
        })
    }

    pub fn record(&mut self, generator: &mut dyn FnMut(usize, i16, i16)) {
        let _ = unsafe { snd_pcm_status(self.sound_device, self.status.as_mut_ptr() as *mut _) };
        let avail = unsafe { snd_pcm_status_get_avail(self.status.as_ptr() as *const _) };

        self.buffer.resize(avail as usize * 2, 0);

        if unsafe { snd_pcm_readi(self.sound_device, self.buffer.as_mut_ptr(), avail as u64) } < 0 {
            println!("Buffer overflow.");
        }

        for i in 0..((avail as usize) / 2) {
            let l = self.buffer[i * 2 as usize];
            let r = self.buffer[i * 2 + 1 as usize];

            generator(0, l, r);
        }
    }
}

impl Drop for Microphone {
    fn drop(&mut self) {
        unsafe {
            snd_pcm_close(self.sound_device);
        }
    }
}
