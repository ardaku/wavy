//! Cross-platform real-time audio recording &amp; playback.
//!
//! The sound waves are _so_ wavy!

#[cfg(any(target_os = "linux", target_os = "android"))]
mod alsa;

pub mod alsa2;

extern crate libc;

use std::collections::VecDeque;

/*/// Resampler for converting to and from sample rates supported by this crates.
pub struct Resampler {
    // Input Hertz
    hz: u32,
    // Output Hertz
    sr: u32,
    // Input Samples
    is: VecDeque<i16>,
    // Resampled audio
    rs: VecDeque<i16>,
    // Count that goes from 0 to the output sample rate and then repeats.
    counter: usize,
    // 
}

impl Resampler {
    /// Create a resampler.
    #[inline(always)]
    pub fn new(hz: u32, sr: u32) -> Resampler {
        let rs = VecDeque::new();
        let is = VecDeque::new();
        let counter = 0;

        Resampler {
            hz, sr, is, rs, counter
        }
    }

    /// Sample at the new `SampleRate`.
    #[inline(always)]
    pub fn play(&mut self, input: &[i16]) {
        // Add Input Samples
        self.is.extend(input);

        // Attempt to Generate output samples
//        let mut cursor = 0;
        loop {
            
            self.counter += 1;
        }
    }

    /// Obtain the resampled audio.
    #[inline(always)]
    pub fn record(&mut self) -> Option<i16> {
        self.rs.pop_front()
    }
}*/

/// 3 sample rates which are supported by this crate.
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum SampleRate {
    /// 24K sample rate.  This is good for where reducing latency matters more than quality.
    Analog = 24_000_u32,
    /// 48K sample rate.  Use this for most things.
    Normal = 48_000_u32,
    /// 96K sample rate.  This is what is recorded in a studio (always downsampled to 48K for
    /// releases though).  Good for when you are slowing down parts of the audio later.
    Studio = 96_000_u32,
}

/// This is the interface to your speaker and microphone.
pub struct AudioIO {
    context: alsa::Context,
    speaker: Option<Speaker>,
    microphone: Option<Microphone>,
}

impl AudioIO {
    /// Create a new Audio Input/Output handle.
    pub fn new(sr: SampleRate) -> AudioIO {
        let context = alsa::Context::new();

        AudioIO {
            speaker: Speaker::new(&context, sr),
            microphone: Microphone::new(&context, sr),
            context,
        }
    }

    /// Play audio from speakers.
    pub fn play(&mut self, generator: &mut FnMut() -> (i16, i16)) {
        if let Some(ref mut speaker) = self.speaker {
            speaker.update(&self.context, generator);
        }
    }

    /// Record audio from microphones.
    pub fn record(&mut self, buffer: &mut [i16]) -> usize {
        if let Some(ref mut microphone) = self.microphone {
            microphone.update(&self.context, buffer)
        } else {
            0
        }
    }
}

impl Drop for AudioIO {
	fn drop(&mut self) {
        if let Some(ref mut microphone) = self.microphone {
    		microphone.pcm.drop(&self.context);
        }
        if let Some(ref mut speaker) = self.speaker {
            speaker.speaker.1.drop(&self.context);
        }
	}
}

fn set_settings(context: &alsa::Context, pcm: &alsa::pcm::PCM, stereo: bool, sr: SampleRate) {
	// Set hardware parameters: 48000 Hz / Mono / 16 bit
	let hwp = alsa::pcm::HwParams::any(context, pcm).unwrap();
	hwp.set_access(context, alsa::pcm::Access::RWInterleaved).unwrap();
	hwp.set_format(context, {
        2
//		if cfg!(target_endian = "little") { 2 }
//		else if cfg!(target_endian = "big") { 3 }
//		else { unreachable!() }
	}).unwrap();
	hwp.set_rate(context, sr as u32, alsa::ValueOr::Nearest).unwrap();
	hwp.set_channels(context, 2 /* if stereo { 2 } else { 1 }*/).unwrap();
    // Make Sure Rate set correctly
	let rate = hwp.get_rate(context).unwrap();
	assert_eq!(rate, sr as u32);
    // Apply and free.
	pcm.hw_params(context, &hwp).unwrap();
	hwp.drop(context);
}

struct Speaker {
	speaker: (usize, alsa::pcm::PCM),
	speaker_buffer: Vec<i16>,
}

impl Speaker {
	/// Connect to a new Speaker.
	pub fn new(context: &alsa::Context, sr: SampleRate) -> Option<Self> {
		let (speaker, speaker_buffer) = {
			let pcm = alsa::pcm::PCM::new(context, "default",
				alsa::Direction::Playback).unwrap();
			set_settings(context, &pcm, true, sr);
			let speaker_max_latency;
			(({
				let hwp = pcm.hw_params_current(context).unwrap();
				let bs = hwp.get_buffer_size(context).unwrap();

				println!("Buffer Size: {}", bs);
				speaker_max_latency
					= hwp.get_period_size(context).unwrap()
						as usize * 2;

				println!("PC: {}", hwp.get_channels(context).unwrap());
				println!("PR: {}", hwp.get_rate(context).unwrap());

				hwp.drop(context);
				bs
			}, pcm), vec![0i16; speaker_max_latency])
		};

		speaker.1.prepare(context);

		Some(Self { speaker, speaker_buffer })
	}

	/// Generate & push data to speaker output.  When a new sample is
	/// needed, closure `generator` will be called.  This should be called
	/// in a loop.
	pub fn update(&mut self, context: &alsa::Context, generator: &mut FnMut() -> (i16, i16)) {
        println!("{}", self.left(context));
		let left = self.left(context) as usize;
		let write = if left < self.speaker_buffer.len() {
			self.speaker_buffer.len() - left
		} else { 0 };

		for i in 0..write {
            let (l, r) = generator();
			self.speaker_buffer[i * 2] = l;
			self.speaker_buffer[i * 2 + 1] = r;
		}

		self.push(context, &self.speaker_buffer[..write]);
	}

	/// Push data to the speaker output.
	fn push(&self, context: &alsa::Context, buffer: &[i16]) {
		if self.speaker.1.writei(context, buffer).unwrap_or_else(|_| {
			0
		}) != buffer.len()
		{
			println!("buffer underrun!");

			self.speaker.1.recover(context, 32, true).unwrap_or_else(|x| {
				panic!("ERROR: {}", x)
			});

			if self.speaker.1.writei(context, buffer).unwrap_or_else(|_| {
				0
			}) != buffer.len() {
				panic!("double buffer underrun!");
			}
		}
	}

	/// Get the number of samples left in the buffer.
	fn left(&self, context: &alsa::Context) -> usize {
		self.speaker.0 - self.speaker.1.status(context).unwrap().get_avail(context)
	}
}

struct Microphone {
	pcm: alsa::pcm::PCM
}

impl Microphone {
	/// Create a new Microphone object.
	pub fn new(context: &alsa::Context, sr: SampleRate) -> Option<Self> {
		let pcm = alsa::pcm::PCM::new(context, "plughw:0,0",
			alsa::Direction::Capture).unwrap();
		set_settings(context, &pcm, true, sr);
		{
			let hwp = pcm.hw_params_current(context).unwrap();
			println!("CC: {}", hwp.get_channels(context).unwrap());
			println!("CR: {}", hwp.get_rate(context).unwrap());
			hwp.drop(context);
		}

		pcm.start(context);

		Some(Self { pcm })
	}

	/// Pull data from the microphone input.
	pub fn update(&self, context: &alsa::Context, buffer: &mut [i16]) -> usize {
		self.pcm.readi(context, buffer).unwrap_or(0)
	}
}
