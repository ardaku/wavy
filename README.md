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
 - Apache License, Version 2.0 ([LICENSE_APACHE_2_0.txt][7]
   or [https://www.apache.org/licenses/LICENSE-2.0][8])
 - Boost License, Version 1.0 ([LICENSE_BOOST_1_0.txt][9]
   or [https://www.boost.org/LICENSE_1_0.txt][10])

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Anyone is more than welcome to contribute!  Don't be shy about getting involved,
whether with a question, idea, bug report, bug fix, feature request, feature
implementation, or other enhancement.  Other projects have strict contributing
guidelines, but this project accepts any and all formats for pull requests and
issues.  For ongoing code contributions, if you wish to ensure your code is
used, open a draft PR so that I know not to write the same code.  If a feature
needs to be bumped in importance, I may merge an unfinished draft PR into it's
own branch and finish it (after a week's deadline for the person who openned
it).  Contributors will always be notified in this situation, and given a choice
to merge early.

All pull request contributors will have their username added in the contributors
section of the release notes of the next version after the merge, with a message
thanking them.  I always make time to fix bugs, so usually a patched version of
the library will be out a few days after a report.  Features requests will not
complete as fast.  If you have any questions, design critques, or want me to
find you something to work on based on your skill level, you can email me at
[jeronlau@plopgrizzly.com](mailto:jeronlau@plopgrizzly.com).  Otherwise,
[here's a link to the issues on GitHub](https://github.com/libcala/wavy/issues),
and, as always, make sure to read and follow the
[Code of Conduct](https://github.com/libcala/wavy/blob/main/CODE_OF_CONDUCT.md).

[0]: https://docs.rs/wavy
[1]: https://crates.io/crates/wavy
[2]: https://github.com/libcala/wavy/actions?query=workflow%3Atests
[3]: https://github.com/libcala/wavy/blob/master/CHANGELOG.md
[4]: https://libcala.github.io/wavy/
[5]: https://github.com/libcala/wavy/
[6]: https://aldaronlau.com/
[7]: https://github.com/libcala/wavy/blob/main/LICENSE_APACHE_2_0.txt
[8]: https://www.apache.org/licenses/LICENSE-2.0
[9]: https://github.com/libcala/wavy/blob/main/LICENSE_BOOST_1_0.txt
[10]: https://www.boost.org/LICENSE_1_0.txt
