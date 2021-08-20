// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

//! Hard-coded constants used throughout the library for dealing with real-time
//! computer audio.

/// Preferred sample rate chosen by this library.  48 KHz is chosen because it
/// is cited as one of the standards for computer audio by
/// [MDN](https://developer.mozilla.org/en-US/docs/Web/Media/Formats/Audio_concepts#Audio_channels_and_frames).
/// The other being 44.1 KHz.  48 KHz is also the default for the Ogg Opus audio
/// format, which is the state-of-the-art audio format.
pub(crate) const SAMPLE_RATE: u16 = 48_000;

/// This is the size of the ring buffer used by the system.
///
/// The number 256 is chosen because it's the minimum allowed by the
/// `createScriptProcessor` JavaScript API, and wavy targets low latency.
pub(crate) const BUFFER_SIZE: u16 = 256;

/// This is the target for how many samples are processed at a time (0.33 ms).
/// This also means that 8 chunks of 32 samples can be contained in a buffer,
/// with buffer size being 256.
///
/// The number 32 is chosen because 0.67 ms latency leaves about 1.33 ms of
/// extra latency before humans would perceive it, which should be enough.
pub(crate) const CHUNK_SIZE: u16 = 32;
