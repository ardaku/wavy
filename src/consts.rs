// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Hard-coded constants used throughout the library for dealing with computer
//! audio.

/// Preferred sample rate chosen by this library.  48 KHz is chosen because it
/// is cited as one of the standards for computer audio by
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/Media/Formats/Audio_concepts#Audio_channels_and_frames).
/// The other being 44.1 KHz.  48 KHz is also the default for the Ogg Opus
/// audio format, which is the state-of-the-art audio format.
pub(crate) const SAMPLE_RATE: u16 = 48_000;

/// The number of audio frames to be sent to/received by hardware at a time.
/// 512 is chosen because it provides a good balance between shortness (about
/// 0.01 seconds) and number of syscalls per second (about 100).  Note that this
/// does not necessarily mean the minimum latency for real time audio is 0.02
/// seconds, because latency is about the synchronization between recording and
/// playback.  It does however mean that the maximum latency for an audio
/// reaction to user input is 11 milliseconds because it's nearly impossible to
/// synchronize the CPU clock with the audio clock without special hardware.
/// That makes the period of 512 frames the largest period at 48 KHz that can
/// react at least as fast as the graphics (which also has it's own clock).
/// Periods should always be powers of 2.
pub(crate) const PERIOD: u16 = 512;
