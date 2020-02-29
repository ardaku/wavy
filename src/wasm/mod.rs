use crate::*;

use stdweb;
use stdweb::js;

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub struct Speaker {
    // sound_device: *mut snd_pcm_t,
    // buffer_size: u64,
    // period_size: u64,
    buffer: Vec<i16>,
}

impl Speaker {
    pub fn new(sr: SampleRate) -> Result<Speaker, AudioError> {
        /*let sound_device: *mut snd_pcm_t = pcm_open(false, b"default\0")?;
        let hw_params = pcm_hw_params(sr, sound_device)?;

        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        //        dbg!(buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(
                hw_params,
                &mut period_size,
                &mut d,
            );
        }
        //        dbg!(period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe { snd_pcm_prepare(sound_device) } < 0 {
            return Err(AudioError::InternalError(
                "Could not prepare!".to_string(),
            ));
        }

        let buffer_size = buffer_size as u64;
        let period_size = period_size as u64;*/

        let buffer = Vec::new();

        Ok(Speaker {
            //sound_device,
            //buffer_size,
            //period_size,
            buffer,
        })
    }

    pub fn play(&mut self, generator: &mut dyn FnMut() -> (i16, i16)) {
        /*let _ = unsafe {
            snd_pcm_status(
                self.sound_device,
                self.status.as_mut_ptr() as *mut _,
            )
        };
        let avail = unsafe {
            snd_pcm_status_get_avail(self.status.as_ptr() as *const _)
        };
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

        if unsafe {
            snd_pcm_writei(
                self.sound_device,
                self.buffer.as_mut_ptr(),
                write as u64,
            )
        } < 0
        {
            println!("Buffer underrun");
        }*/
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        /*unsafe {
            snd_pcm_close(self.sound_device);
        }*/
    }
}

pub struct Microphone {
    // sound_device: *mut snd_pcm_t,
    buffer: Vec<i16>,
}

impl Microphone {
    pub fn new(sr: SampleRate) -> Result<Microphone, AudioError> {
        /*        let sound_device: *mut snd_pcm_t = pcm_open(true, b"default\0")?;
        let hw_params = pcm_hw_params(sr, sound_device)?;

        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        //        dbg!(buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(
                hw_params,
                &mut period_size,
                &mut d,
            );
        }
        //        dbg!(period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe { snd_pcm_start(sound_device) } < 0 {
            return Err(AudioError::InternalError(
                "Could not start!".to_string(),
            ));
        }*/

        let buffer = Vec::new();

        Ok(Microphone {
            // sound_device,
            buffer,
        })
    }

    pub fn record(&mut self, generator: &mut dyn FnMut(usize, i16, i16)) {
        /*let _ = unsafe {
            snd_pcm_status(
                self.sound_device,
                self.status.as_mut_ptr() as *mut _,
            )
        };
        let avail = unsafe {
            snd_pcm_status_get_avail(self.status.as_ptr() as *const _)
        };

        self.buffer.resize(avail as usize * 2, 0);

        if unsafe {
            snd_pcm_readi(
                self.sound_device,
                self.buffer.as_mut_ptr(),
                avail as u64,
            )
        } < 0
        {
            println!("Buffer overflow.");
        }

        for i in 0..((avail as usize) / 2) {
            let l = self.buffer[i * 2 as usize];
            let r = self.buffer[i * 2 + 1 as usize];

            generator(0, l, r);
        }*/
    }
}

impl Drop for Microphone {
    fn drop(&mut self) {
        /*unsafe {
            snd_pcm_close(self.sound_device);
        }*/
    }
}
