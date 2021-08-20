// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::consts::CHUNK_SIZE;
use crate::raw::{global, Speakers as RawSpeakers};
use fon::chan::{Ch32, Channel};
use fon::{Frame, Sink};
use std::fmt::{Debug, Display, Formatter, Result};
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll};

/// Play audio samples through a speaker.
///
/// # 440 HZ Sine Wave Example
/// **note:** This example depends on `twang = "0.5"` to synthesize the sine
/// wave.
/// ```no_run
/// use fon::{stereo::Stereo32, Sink};
/// use pasts::{exec, wait};
/// use twang::{Fc, Signal, Synth};
/// use wavy::{Speakers, SpeakersSink};
///
/// /// An event handled by the event loop.
/// enum Event<'a> {
///     /// Speaker is ready to play more audio.
///     Play(SpeakersSink<'a, Stereo32>),
/// }
///
/// /// Shared state between tasks on the thread.
/// struct State {
///     /// A streaming synthesizer using Twang.
///     synth: Synth<()>,
/// }
///
/// impl State {
///     /// Event loop.  Return false to stop program.
///     fn event(&mut self, event: Event<'_>) {
///         match event {
///             Event::Play(mut speakers) => speakers.stream(&mut self.synth),
///         }
///     }
/// }
///
/// /// Program start.
/// fn main() {
///     fn sine(_: &mut (), fc: Fc) -> Signal {
///         fc.freq(440.0).sine().gain(0.7)
///     }
///
///     let mut state = State { synth: Synth::new((), sine) };
///     let mut speakers = Speakers::default();
///
///     exec!(state.event(wait! {
///         Event::Play(speakers.play().await),
///     }));
/// }
/// ```
pub struct Speakers(Rc<dyn RawSpeakers>);

impl Speakers {
    /// Create a new speaker from a native microphone.
    pub(crate) fn new<N: RawSpeakers + 'static>(native: N) -> Self {
        Self(Rc::new(native))
    }

    /// Query available audio destinations.
    pub fn query() -> impl Iterator<Item = Self> {
        global().query_speakers().map(|x| Self(x))
    }
}

impl Display for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Speakers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Speakers({})", self.0)
    }
}

impl Default for Speakers {
    fn default() -> Self {
        Self::query().next().unwrap()
    }
}

impl Future for Speakers {
    type Output = Player;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(()) = self.0.poll(cx) {
            Poll::Ready(Player(Rc::downgrade(&self.0)))
        } else {
            Poll::Pending
        }
    }
}

/// Audio player.
pub struct Player(Weak<dyn RawSpeakers>);

impl Debug for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Player({})", self.0.upgrade().unwrap())
    }
}

impl<const N: usize> Sink<Ch32, N> for Player {
    fn sample_rate(&self) -> NonZeroU32 {
        NonZeroU32::new(self.0.upgrade().unwrap().sample_rate()).unwrap()
    }

    fn len(&self) -> usize {
        CHUNK_SIZE.into()
    }

    fn sink_with<I: Iterator<Item = Frame<Ch32, N>>>(&mut self, iter: I) {
        let system = self.0.upgrade().unwrap();
        let samples = system.play();
        for (i, frame) in iter.enumerate() {
            for (j, chan) in frame.channels().iter().cloned().enumerate() {
                samples[j][i] = chan.to_f32();
            }
        }
    }
}
