// Sealed trait for Frames
pub trait Frame {
    // Hertz
    const HZ: u32;
    // Channel count
    const CH: u8;
    // Type for a sample
    type Sample;
}
