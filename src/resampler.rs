

/// Resampler for converting to and from sample rates supported by this crates.
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

    /// Sample at the new sample rate.
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
}
