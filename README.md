# Wavy

#### [Changelog][3] | [Source][4] | [Getting Started][5]

[![tests](https://github.com/libcala/wavy/workflows/tests/badge.svg)][2]
[![docs](https://docs.rs/wavy/badge.svg)][0]
[![crates.io](https://img.shields.io/crates/v/wavy.svg)][1]

The sound waves are _so_ wavy!

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
Licensed under any of
 - Apache License, Version 2.0, ([LICENSE_APACHE_2_0.txt][7]
   or [https://www.apache.org/licenses/LICENSE-2.0][8])
 - MIT License, ([LICENSE_MIT.txt][9] or [https://mit-license.org/][10])
 - Boost Software License, Version 1.0, ([LICENSE_BOOST_1_0.txt][11]
   or [https://www.boost.org/LICENSE_1_0.txt][12])

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as described above, without any additional terms or conditions.

## Help
If you want help using or contributing to this library, feel free to send me an
email at [aldaronlau@gmail.com][13].

[0]: https://docs.rs/wavy
[1]: https://crates.io/crates/wavy
[2]: https://github.com/libcala/wavy/actions?query=workflow%3Atests
[3]: https://github.com/libcala/wavy/blob/master/CHANGELOG.md
[4]: https://github.com/libcala/wavy/
[5]: https://docs.rs/wavy#getting-started
[6]: https://aldaronlau.com/
[7]: https://github.com/libcala/wavy/blob/main/LICENSE_APACHE_2_0.txt
[8]: https://www.apache.org/licenses/LICENSE-2.0
[9]: https://github.com/libcala/wavy/blob/main/LICENSE_MIT.txt
[10]: https://mit-license.org/
[11]: https://github.com/libcala/wavy/blob/main/LICENSE_BOOST_1_0.txt
[12]: https://www.boost.org/LICENSE_1_0.txt
[13]: mailto:aldaronlau@gmail.com
