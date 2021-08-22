// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use flume::Sender;

// The asynchronous task that handles finding speakers and microphones.
async fn audio_task(
    speaker_broadcaster: Sender<crate::Speakers>,
    microphone_broadcaster: Sender<crate::Microphone>,
) {
}

/// Start the audio thread for the Linux platform.
pub(super) fn start(
    speaker_broadcaster: Sender<crate::Speakers>,
    microphone_broadcaster: Sender<crate::Microphone>,
) {
    std::thread::spawn(|| {
        pasts::block_on(async {
            audio_task(speaker_broadcaster, microphone_broadcaster)
        })
    });
}
