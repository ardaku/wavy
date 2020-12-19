// Copyright Jeron Aldaron Lau 2019 - 2020.
// Distributed under either the Apache License, Version 2.0
//    (See accompanying file LICENSE_APACHE_2_0.txt or copy at
//          https://apache.org/licenses/LICENSE-2.0),
// or the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE_BOOST_1_0.txt or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
// at your option. This file may not be copied, modified, or distributed except
// according to those terms.

use std::fmt::{Display, Error, Formatter};

pub(crate) trait SoundDevice: Display {
    const INPUT: bool;
}

#[derive(Debug, Default)]
pub(crate) struct AudioSrc();

impl SoundDevice for AudioSrc {
    const INPUT: bool = true;
}

impl Display for AudioSrc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Default")
    }
}

#[derive(Debug, Default)]
pub(crate) struct AudioDst();

impl SoundDevice for AudioDst {
    const INPUT: bool = false;
}

impl Display for AudioDst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Default")
    }
}

/// Return a list of available audio devices.
pub(crate) fn device_list<D: SoundDevice, F: Fn(D) -> T, T>(
    _abstrakt: F,
) -> Vec<T> {
    vec![]
}
