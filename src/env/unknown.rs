// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

compile_error!(concat!(
    "Target environment ",
    include_str!(concat!(env!("OUT_DIR"), "/target")),
    " not supported, please open an issue at \
    https://github.com/libcala/wavy/issues"
));

// Reduce errors by defining expected `start()` function.
use flume::Sender;
pub(super) fn start(_: Sender<crate::Speakers>, _: Sender<crate::Microphone>) {}
