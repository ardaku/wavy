// Copyright © 2019-2021 The Wavy Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! This module contains all of the environment-specific code in this crate.
//! This module makes use of channels for inter-task communication, starting up
//! the platform-specific code as it's own task.  Depending on the platform, the
//! task may be implemented as a thread or through some asynchronous system.
//!
//! This module (including submodules) is the only part of this crate where
//! unsafe code is allowed, as it's necessary for most FFI.
//!
//!  1. Linux: start smelling_salts thread running an epoll loop on ALSA devices
//!  2. MacOS: start RunLoop for AudioQueue FIXME
//!  3. Web: start audio web worker FIXME
//!    - Web (fallback): enable asynchronous audio callbacks
//!  4. Windows: TBD FIXME
//!  5. Ardaku: TBD FIXME
//!  6. BSD: TBD FIXME
//!  7. Android: TBD FIXME
//!  8. Fuchsia: TBD FIXME
//!  9. iOS: TBD (probably same as MacOS) FIXME
//!  10. Redox: TBD FIXME
//!  11. Xbox: TBD FIXME
//!  12. PlayStation: TBD FIXME
//!  13. Nintendo: TBD FIXME
//!  14. Others?

#![allow(unsafe_code)]

use fon::chan::Ch32;
use fon::Frame;
use std::mem::MaybeUninit;
use std::num::NonZeroU32;
use std::sync::Once;

/// Import a different module depending on target environment.
#[cfg_attr(
    any(target_family = "wasm", target_arch = "wasm32"),
    cfg_attr(target_os = "ardaku", path = "env/ardaku.rs"),
    cfg_attr(
        any(target_os = "unknown", target_os = "emscripten"),
        path = "env/web.rs"
    )
)]
#[cfg_attr(
    not(any(target_family = "wasm", target_arch = "wasm32")),
    cfg_attr(target_os = "linux", path = "env/linux.rs"),
    cfg_attr(target_os = "android", path = "env/android.rs"),
    cfg_attr(target_os = "macos", path = "env/macos.rs"),
    cfg_attr(target_os = "ios", path = "env/ios.rs"),
    cfg_attr(target_os = "windows", path = "env/windows.rs"),
    cfg_attr(target_os = "fuchsia", path = "env/fuchsia.rs"),
    cfg_attr(target_os = "redox", path = "env/redox.rs"),
    cfg_attr(
        any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "bitrig",
            target_os = "openbsd",
            target_os = "netbsd"
        ),
        path = "env/bsd.rs",
    )
)]
mod unknown;

/// Environment-Sent Chunk Playback Event.
pub struct Playback {
    /// Hardware sample rate.
    sample_rate: NonZeroU32,

    /// Function to write & return audio data buffer back to the speakers.
    callback: fn(&mut dyn Iterator<Item = Frame<Ch32, 8>>),
}

impl Playback {
    /// Consume playback event, and write out audio data.
    pub(crate) fn play(&self, iter: &mut dyn Iterator<Item = Frame<Ch32, 8>>) {
        (self.callback)(iter);
    }

    /// Get the sample rate of the speakers.
    pub(crate) fn sample_rate(&self) -> NonZeroU32 {
        self.sample_rate
    }
}

/// Environment-Sent Chunk Recording Event.
pub struct Recording {
    /// Hardware sample rate.
    sample_rate: NonZeroU32,

    /// Function to read audio data buffer from the microphone.
    callback: fn(&mut dyn FnMut(&mut dyn Iterator<Item = Frame<Ch32, 8>>)),
}

impl Recording {
    /// Consume record event, and read in audio frames with a callback.
    pub(crate) fn record(
        &self,
        func: &mut dyn FnMut(&mut dyn Iterator<Item = Frame<Ch32, 8>>),
    ) {
        (self.callback)(func);
    }

    /// Get the sample rate of the microphone.
    pub(crate) fn sample_rate(&self) -> NonZeroU32 {
        self.sample_rate
    }
}

/// Global state shared between all threads.
struct State {
    speaker_listener: Receiver<crate::Speakers>,
    microphone_listener: Receiver<crate::Microphone>,
}

/// Once to initialize the state.
static START: Once = Once::new();

/// Global state shared between all threads.
static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

/// Safely access the global state.
fn with_state<T, F: FnOnce(&State) -> T>(f: F) -> T {
    // Create channels for microphone and speaker connections.
    let (microphone_broadcaster, microphone_listener) = flume::bounded(1);
    let (speaker_broadcaster, speaker_listener) = flume::bounded(1);

    // *unsafe*: `static mut` accessed in synchronized fashion.
    unsafe {
        // Make sure initialized
        START.call_once(move || {
            STATE = MaybeUninit::new(State {
                speaker_listener,
                microphone_listener,
            });
            unknown::start(speaker_broadcaster, microphone_broadcaster);
        });
        // Use closure to guarantee limited (sound) lifetime.
        f(&*STATE.as_ptr())
    }
}

pub(crate) fn query_speakers() -> Receiver<crate::Speakers> {
    with_state(|s| s.speaker_listener.clone())
}

pub(crate) fn query_microphones() -> Receiver<crate::Microphone> {
    with_state(|s| s.microphone_listener.clone())
}
