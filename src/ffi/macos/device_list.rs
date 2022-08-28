// Copyright © 2019-2022 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// - MIT License (https://mit-license.org/)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::fmt::Display;

pub(crate) trait SoundDevice: Display {
    const INPUT: bool;
}

/// Return a list of available audio devices.
pub(crate) fn device_list<D: SoundDevice, F: Fn(D) -> T, T>(
    _abstrakt: F,
) -> Vec<T> {
    vec![]
}
