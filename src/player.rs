use std::iter::IntoIterator;
use std::borrow::Borrow;

/// Audio (Speaker) output.  This type represents a speaker system.
pub struct Player(pub(crate) crate::ffi::Player);

impl Player {
    /// Connect to the speaker system at a specific sample rate.
    pub fn new(
        sr: crate::SampleRate,
    ) -> Result<Player, crate::AudioError> {
        Ok(Player(crate::ffi::Player::new(sr)?))
    }

    /// Generate audio samples as they are needed.  In your closure return S16_LE audio samples.
    pub async fn play_last<T>(&mut self, iter: impl IntoIterator<Item=T>) -> Result<usize, crate::AudioError>
    where
        T: Borrow<crate::StereoS16Frame>,
    {
        self.0.play_last(iter).await
    }
}
