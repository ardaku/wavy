use crate::frame::Frame;

/// Frame with Stereo Signed 16-Bit Little Endian format.  Always 32 bits.
#[repr(C)]
#[repr(align(4))]
#[derive(Copy, Clone, Debug)]
pub struct StereoS16 {
    /// Always stored as Little Endian
    left: i16,
    /// Always stored as Little Endian
    right: i16,
}

impl Frame for StereoS16 {
    // Hertz
    const HZ: u32 = 48_000;
    // Channel count
    const CH: u8 = 2;
    // Type for a sample
    type Sample = i16;
}

impl StereoS16 {
    /// Create a new StereoS16 from sample in the target platform's native
    /// endian.
    pub const fn new(left: i16, right: i16) -> Self {
        StereoS16 {
            left: left.to_le(),
            right: right.to_le(),
        }
    }

    /// Get the left sample in the target platform's native endian.
    pub fn left(self) -> i16 {
        i16::from_le(self.left)
    }

    /// Get the right sample in the target platform's native endian.
    pub fn right(self) -> i16 {
        i16::from_le(self.right)
    }

    /// Get a byte representation of this frame.
    pub fn bytes(self) -> [u8; 4] {
        let [a, b] = self.left.to_ne_bytes();
        let [c, d] = self.right.to_ne_bytes();

        [a, b, c, d]
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    fn sizes() {
        assert_eq!(std::mem::size_of::<StereoS16>(), 4);
    }
}
