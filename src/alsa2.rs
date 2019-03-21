#![allow(non_camel_case_types)]

use crate::SampleRate;

use libc;
use libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void, size_t};

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

    fn snd_pcm_hw_params_set_rate_resample(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t,
        a: c_uint) -> c_int;

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

    fn snd_pcm_hw_params_get_buffer_size(params: *const snd_pcm_hw_params_t, val: *mut c_ulong)
        -> c_int;

    fn snd_pcm_hw_params_get_period_size(
        params: *const snd_pcm_hw_params_t,
        frames: *mut c_ulong,
        dir: *mut c_int,
    ) -> c_int;

    fn snd_pcm_hw_params_free(obj: *mut snd_pcm_hw_params_t);

    fn snd_pcm_prepare(pcm: *mut snd_pcm_t) -> c_int;

/*    fn snd_pcm_writei(
        pcm: *mut snd_pcm_t,
        buffer: *const i16,
        size: c_ulong,
    ) -> snd_pcm_sframes_t;*/

    fn snd_pcm_writen(
        pcm: *mut snd_pcm_t,
        buffers: *mut *mut i16,
        size: c_ulong,
    ) -> snd_pcm_sframes_t;

    fn snd_pcm_status_sizeof() -> size_t;

    fn snd_pcm_status(pcm: *mut snd_pcm_t, status: *mut snd_pcm_status_t) -> c_int;

    fn snd_pcm_status_get_avail(obj: *const snd_pcm_status_t) -> c_ulong;
}

/// Audio (Speaker) output.
pub struct Speaker {
    sound_device: *mut snd_pcm_t,
    buffer_size: u64,
    period_size: u64,
    status: Vec<u8>,
    lbuffer: Vec<i16>,
    rbuffer: Vec<i16>,
}

impl Speaker {
    /// Connect to speaker at a specific sample rate.
    pub fn new(sr: SampleRate) -> Speaker {
        let mut sound_device: *mut snd_pcm_t = unsafe { std::mem::uninitialized() };
        if unsafe {
            snd_pcm_open(&mut sound_device, b"plughw:0,0\0".as_ptr() as *const _, 0 /*playback*/, 0)
        } < 0 {
            panic!("Failed to open audio device!");
        }
        let mut hw_params: *mut snd_pcm_hw_params_t = unsafe { std::mem::uninitialized() };
        if unsafe {
            snd_pcm_hw_params_malloc(&mut hw_params)
        } < 0 {
            panic!("Cannot allocate hardware parameter structure!");
        }
        if unsafe {
            snd_pcm_hw_params_any(sound_device, hw_params)
        } < 0 {
            panic!("Cannot initialize hardware parameter structure!");
        }
        // Enable resampling.
        if unsafe {
            snd_pcm_hw_params_set_rate_resample(sound_device, hw_params, 1)
        } < 0 {
            panic!("Resampling setup failed for playback: ");
        }
        // Set access to RW noninterleaved.
        if unsafe {
            snd_pcm_hw_params_set_access(sound_device, hw_params, 4 /*RW_NONINTERLEAVED*/)
        } < 0 {
            panic!("Cannot set access type!");
        }
        // 
        if unsafe {
            snd_pcm_hw_params_set_format(sound_device, hw_params, 2 /*S16_LE*/)
        } < 0 {
            panic!("Cannot set sample format!");
        }
        // Set channels to stereo (2).
        if unsafe {
            snd_pcm_hw_params_set_channels(sound_device, hw_params, 2)
        } < 0 {
            panic!("Cannot set channel count!");
        }
        // Set Sample rate.
        let mut actual_rate = sr as u32;
        if unsafe {
            snd_pcm_hw_params_set_rate_near(sound_device, hw_params, &mut actual_rate, std::ptr::null_mut())
        } < 0 {
            panic!("Cannot set sample rate!");
        }
        if actual_rate != sr as u32 {
            panic!("Failed to set rate: {}, Got: {} instead!", sr as u32, actual_rate);
        }
        // Apply the hardware parameters that just got set.
        if unsafe {
            snd_pcm_hw_params(sound_device, hw_params)
        } < 0 {
            panic!("Failed to set parameters!");
        }
        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        println!("Buffer Size: {}", buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(hw_params, &mut period_size, &mut d);
        }
        println!("Period is {} frames", period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe {
            snd_pcm_prepare(sound_device)
        } < 0 {
            panic!("Could not prepare!");
        }

/*        let mut buf = [0i16; 128];

        for i in 0..64 {
            buf[i as usize] = i * 255 * 2;
            buf[i as usize + 64] = 32767 - buf[i as usize];
        }

        for i in 0..100 {
            unsafe {
                snd_pcm_writei(sound_device, buf.as_ptr(), buf.len() as u64);
            }
        }

        loop {}*/

        let buffer_size = buffer_size as u64;
        let period_size = period_size as u64;

        let status = vec![0; unsafe { snd_pcm_status_sizeof() }];
        let (lbuffer, rbuffer) = (Vec::new(), Vec::new());

        Speaker {
            sound_device, buffer_size, status, period_size, lbuffer, rbuffer,
        }

/*  snd_pcm_uframes_t bufferSize;
  
  // If we were going to do more with our sound device we would want to store
  // the buffer size so we know how much data we will need to fill it with.
  cout << "Init: Buffer size = " << bufferSize << " frames." << endl;

  // Display the bit size of samples.
  cout << "Init: Significant bits for linear samples = " << snd_pcm_hw_params_get_sbits(hw_params) << endl;

  // Free the hardware parameters now that we're done with them.
  snd_pcm_hw_params_free (hw_params);

  // Prepare interface for use.
  if ((err = snd_pcm_prepare (_soundDevice)) < 0)
  {
      cout << "Init: cannot prepare audio interface for use (" << snd_strerror (err) << ")" << endl;
      return false;
  }
  else
  {
      cout << "Audio device has been prepared for use." << endl;
  }*/

/*  short buf[128];

  for(int i = 0; i < 64; i++) {
	  buf[i] = i * 255 * 2;
	  buf[i + 64] = 65535 - buf[i];
  }

  for(int i = 0; i < 100; i++) {
  	snd_pcm_writei(_soundDevice, buf, 128);
  }

  while(true) { }

  return true;*/
    }

    pub fn play(&mut self, generator: &mut FnMut() -> (i16, i16)) {
        let status = unsafe { snd_pcm_status(self.sound_device, self.status.as_mut_ptr() as *mut _) };
        let avail = unsafe { snd_pcm_status_get_avail(self.status.as_ptr() as *const _) };
        let left = self.buffer_size - avail;

        println!("{}", left);

        let buffer_length = self.period_size * 4; // 16 bit (2 bytes) * Stereo (2 channels)

        let write = if left < buffer_length {
			buffer_length - left
		} else { 0 };

        self.lbuffer = Vec::new();
        self.rbuffer = Vec::new();

		for _i in 0..write {
            let (l, r) = generator();
			self.lbuffer.push(l);
			self.rbuffer.push(r);
		}

        if unsafe {
            snd_pcm_writen(self.sound_device, [self.lbuffer.as_mut_ptr(), self.rbuffer.as_mut_ptr()].as_mut_ptr(), write as u64)
//            snd_pcm_writei(self.sound_device, self.buffer.as_ptr(), write as u64)
        } < 0 {
            println!("Buffer underrun (probably).");
        }

//		self.push(context, &self.buffer[..write]);
    }
}

/// Audio (Microphone) Input.
pub struct Microphone {
    
}
