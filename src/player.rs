use std::iter::IntoIterator;
use std::borrow::Borrow;

/// Audio Player (Speaker output). 
pub struct Player(pub(crate) crate::ffi::Player);

impl Player {
    /// Connect to the speaker system at a specific sample rate.
    pub fn new(
        sr: crate::SampleRate,
    ) -> Result<Player, crate::AudioError> {
        Ok(Player(crate::ffi::Player::new(sr)?))
    }

    /// Play audio samples from an iterator.  Get a future that returns the
    /// number of audio samples played.
    pub async fn play_last<T>(&mut self, iter: impl IntoIterator<Item=T>) -> usize
    where
        T: Borrow<crate::StereoS16Frame>,
    {
        self.0.play_last(iter).await
    }
}
