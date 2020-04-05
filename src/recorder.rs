use crate::ffi;

/// Audio Recorder (Microphone input).
pub struct Recorder(ffi::Recorder);

impl Recorder {
    /// Create a new audio recorder at a specific sample rate.
    pub fn new(sr: crate::SampleRate) -> Option<Recorder> {
        Some(Recorder(ffi::Recorder::new(sr)?))
    }

    /// Record audio from connected microphones.  Get a future that returns
    /// a slice of the newly recorded audio frames.
    pub async fn record_last(&mut self) -> &[crate::S16LEx2] {
        self.0.record_last().await
    }
}
