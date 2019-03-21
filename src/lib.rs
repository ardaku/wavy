//! Cross-platform real-time audio recording &amp; playback.
//!
//! The sound waves are _so_ wavy!

pub mod prelude;
mod error;
mod sample_rate;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod alsa;

mod linux;

extern crate libc;

pub use error::AudioError;
pub use sample_rate::SampleRate;

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

/*fn set_settings(context: &alsa::Context, pcm: &alsa::pcm::PCM, stereo: bool, sr: SampleRate) {
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
}*/

/// Audio (Speaker) output.  This type represents a speaker system.
pub struct SpeakerSystem(
    #[cfg(target_os = "linux")]
    linux::Speaker,
);

impl SpeakerSystem {
    /// Connect to speaker at a specific sample rate.
    pub fn new(sr: SampleRate) -> Result<SpeakerSystem, AudioError> {
        Ok(SpeakerSystem(
            #[cfg(target_os = "linux")] {
                linux::Speaker::new(sr)?
            },
        ))
    }

    /// Generate audio samples as they are needed.  In your closure return 2 U16_LE audio samples.
    /// Stereo Audio - Left, Right.
    pub fn play(&mut self, generator: &mut FnMut() -> (i16, i16)) {
        self.0.play(generator)
    }

    /// Generate audio samples as they are needed.  In your closure return 2 U16_LE audio samples.
    /// 5.1 surround sound audio - Front Left, Front Right, Center, Rear Left, Rear Right, Bass.
    pub fn play_ss(&mut self, generator: &mut FnMut() -> (i16, i16, i16, i16, i16, i16)) {
        // TODO: Right now we're just combining into a stereo track for playback whether or not we
        // have 5.1 support.
        self.0.play(&mut || {
            let (front_l, front_r, center, rear_l, rear_r, bass) = generator();
            let l = front_l >> 2 + rear_l >> 2 + center >> 2 + bass >> 2;
            let r = front_r >> 2 + rear_r >> 2 + center >> 2 + bass >> 2;
            (l, r)
        })
    }
}

/// Audio (Microphone) input.
pub struct MicrophoneSystem(
    #[cfg(target_os = "linux")]
    linux::Microphone,
);

impl MicrophoneSystem {
	/// Create a new Microphone object.
	pub fn new(sr: SampleRate) -> Result<MicrophoneSystem, AudioError> {
        Ok(MicrophoneSystem(
            #[cfg(target_os = "linux")] {
                linux::Microphone::new(sr)?
            },
        ))
	}

	/// Record audio from the microphone system.  The closures first parameter is the microphone id.
    /// The 2nd and 3rd are left and right sample.
	pub fn record(&mut self, generator: &mut FnMut(usize, i16, i16)) {
        self.0.record(generator);
	}
}
