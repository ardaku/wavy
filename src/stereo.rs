use crate::frame::Frame;

/// Frame with Stereo Signed 16-Bit Little Endian format.  Always 32 bits.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default)]
pub struct S16LEx2([u8; 4]);

impl Frame for S16LEx2 {
    // Channel count
    const CH: u8 = 2;
    // Type for a sample
    type Sample = i16;
}

impl S16LEx2 {
    /// Create a new frame from sample in the target platform's native endian.
    pub fn new(left: i16, right: i16) -> Self {
        let left = left.to_le_bytes();
        let right = right.to_le_bytes();

        Self([left[0], left[1], right[0], right[1]])
    }

    /// Get the left sample in the target platform's native endian.
    pub fn left(self) -> i16 {
        i16::from_le_bytes([self.0[0], self.0[1]])
    }

    /// Get the right sample in the target platform's native endian.
    pub fn right(self) -> i16 {
        i16::from_le_bytes([self.0[2], self.0[3]])
    }

    /// Get a byte representation of this frame.
    pub fn bytes(self) -> [u8; 4] {
        self.0
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    fn sizes() {
        assert_eq!(std::mem::size_of::<S16LEx2>(), 4);
    }
}
