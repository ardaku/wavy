// Sealed trait for Frames
pub trait Frame: Copy + Clone {
    // Channel count
    const CH: u8;
    // Type for a sample
    type Sample;
}
