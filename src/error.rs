/// Error for opening audio device for reading (recording) or writing (playing).
#[derive(Debug)]
pub enum AudioError {
    /// There is no speaker/microphone.
    NoDevice,
    /// Generally, this shouldn't happen.  If it does open an issue on GitHub (probably a bug).
    InternalError(String),
}
