use crate::ffi;
use crate::frame::Frame;

/// Audio Recorder (Microphone input).
pub struct Recorder<F: Frame>(ffi::Recorder<F>);

impl<F: Frame> Recorder<F> {
    /// Create a new audio recorder at a specific sample rate.
    pub fn new(sr: u32) -> Option<Self> {
        Some(Recorder(ffi::Recorder::new(sr)?))
    }

    /// Record audio from connected microphones.  Get a future that returns
    /// a slice of the newly recorded audio frames.
    pub async fn record_last(&mut self, audio: &mut Vec<F>) {
        // This checks to see if any samples can be added (capacity is used).
        // If not, reserve space.
        if audio.len() + 1024 > audio.capacity() {
            audio.reserve(audio.capacity() + 1024);
        }
        self.0.record_last(audio).await
    }
}
