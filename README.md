# Wavy

#### The sound waves are _so_ wavy!

[![tests](https://github.com/libcala/wavy/workflows/tests/badge.svg)][2]
[![docs](https://docs.rs/wavy/badge.svg)][0]
[![crates.io](https://img.shields.io/crates/v/wavy.svg)][1]

[About][4] | [Source][5] | [Changelog][3] | [Tutorial][6]

## About
Library for asynchronous cross-platform real-time audio recording &amp;
playback.  This library is great for if you need low-latency sound effects in
video games, if you're making a multi-media player, Digital Audio
Workstation, or building a synthesizer; anything that needs access to speakers
or microphones.

Check out the [documentation][0] for examples.

### Supported Platforms
Wavy targets all platforms that can run Rust.
 - Linux/**Android Untested** (Using ALSA C Library)
 - Web (Using JavaScript's Web Audio API)
 - MacOS/iOS **WIP** (Using AudioQueue C Library)
 - Windows **Planned Next, after MacOS**

## License
Licensed under either of
 - Apache License, Version 2.0,
   ([LICENSE-APACHE][7] or [https://www.apache.org/licenses/LICENSE-2.0][8])
 - Zlib License,
   ([LICENSE-ZLIB][9] or [https://opensource.org/licenses/Zlib][10])
at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[0]: https://docs.rs/wavy
[1]: https://crates.io/crates/wavy
[2]: https://github.com/libcala/wavy/actions?query=workflow%3Atests
[3]: https://github.com/libcala/wavy/blob/master/CHANGELOG.md
[4]: https://libcala.github.io/wavy/
[5]: https://github.com/libcala/wavy/
[6]: https://aldaronlau.com/
[7]: https://github.com/libcala/wavy/blob/master/LICENSE-APACHE
[8]: https://www.apache.org/licenses/LICENSE-2.0
[9]: https://github.com/libcala/wavy/blob/master/LICENSE-ZLIB
[10]: https://opensource.org/licenses/Zlib
[11]: mailto:jeronlau@plopgrizzly.com
