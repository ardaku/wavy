/// Audio Recorder (Microphone input).
pub struct Recorder(crate::ffi::Recorder);

impl Recorder {
    /// Connect to the microphone system at a specific sample rate.
    pub fn new(
        sr: crate::SampleRate,
    ) -> Result<Recorder, crate::AudioError> {
        Ok(Recorder(crate::ffi::Recorder::new(sr)?))
    }

    /// Record audio from the microphone system.  The closures first parameter is the microphone id.
    /// The 2nd and 3rd are left and right sample.
    pub async fn record_last(&mut self) -> Result<&[crate::StereoS16Frame], crate::AudioError> {
        self.0.record_last().await
    }
}
