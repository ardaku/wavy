use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ffi;
use crate::frame::Frame;

/// Audio Player (Speaker output).
pub struct Player<F: Frame>(ffi::Player<F>);

impl<F: Frame> Player<F> {
    /// Connect to the speaker system at a specific sample rate.
    pub fn new(sr: u32) -> Option<Player<F>> {
        Some(Player(ffi::Player::new(sr)?))
    }

    /// Play from a slice of audio samples.  Returns a future that returns the
    /// number of audio samples actually played.
    pub fn play_last(&mut self, audio: &[F]) -> usize {
        self.0.play_last(audio)
    }
}

impl<F: Frame + Unpin> Future for Player<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.get_mut().0.poll(cx)
    }
}
