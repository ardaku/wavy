// Wavy
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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
    
    fn into_f64x2(self) -> (f64, f64) {
        (self.left() as f64 / i16::MAX as f64, self.right() as f64 / i16::MAX as f64)
    }
    
    fn from_f64x2(left: f64, right: f64) -> Self {
        Self::new((left * i16::MAX as f64) as i16, (right * i16::MAX as f64) as i16)
    }
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
