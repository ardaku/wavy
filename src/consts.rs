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

/// Set latency to be about 1 millisecond.  This is how many samples need to be
/// generated at each call to/from microphone or speaker.
///
/// 48 is the minimum period that doesn't create bad-sounding artifacts on ALSA
/// running on PulseAudio.
pub(crate) const PERIOD: u16 = 48;
