// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

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
/// running on PulseAudio for my testing, bumped up to 64 (1.5 ms) should be
/// sufficient.  Humans generally can't tell at about 2 ms, which gives .5ms
/// leeway for processing time.
pub(crate) const PERIOD: u16 = 64;
