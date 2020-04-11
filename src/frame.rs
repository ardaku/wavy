// Sealed trait for Frames
pub trait Frame: Copy + Clone + Default {
    // Channel count
    const CH: u8;
    // Type for a sample
    type Sample;
}
