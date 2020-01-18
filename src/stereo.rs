/// Frame with Stereo Signed 16-Bit Little Endian format.  Always 32 bits.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StereoS16Frame {
    left: i16,
    right: i16,
}

impl StereoS16Frame {
    /// Create a new StereoS16Frame from sample in the target platform's native
    /// endian.
    pub const fn new(left: i16, right: i16) -> Self {
        StereoS16Frame {
            left: left.to_le(),
            right: right.to_le(),
        }
    }

    /// Get the left sample in the target platform's native endian.
    pub fn left(&self) -> i16 {
        i16::from_le(self.left)
    }

    /// Get the right sample in the target platform's native endian.
    pub fn right(&self) -> i16 {
        i16::from_le(self.right)
    }
}

#[cfg(tests)]
mod tests {
    #[test]
    fn sizes() {
        assert_eq!(std::mem::size_of::<StereoS16Frame>(), 4);
    }
}
