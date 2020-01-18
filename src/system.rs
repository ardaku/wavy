//! Audio System (SpeakerList & MicrophoneList).

// FIXME: Probably remove

/// An AudioSample (with surround sound 5.1 support).
pub struct AudioSample {
    front_left: i16,
    front_right: i16,
    #[allow(unused)] // TODO
    center: i16,
    #[allow(unused)] // TODO
    lfe: i16,
    surround_left: i16,
    surround_right: i16,
}

impl AudioSample {
    /// Create stereo audio sample.
    pub fn stereo(left: i16, right: i16) -> AudioSample {
        AudioSample {
            front_left: left,
            front_right: right,
            center: 0,
            lfe: 0,
            surround_left: left,
            surround_right: right,
        }
    }

    /// Create surround sound 5.1 audio sample.
    /// * Center: 0°.
    /// * Front-Left: -30°
    /// * Front-Right: 30°
    /// * Surround-Left: -110°
    /// * Surround-Right: 110°
    ///
    /// _source:_ [https://en.wikipedia.org/wiki/5.1_surround_sound#Music](https://en.wikipedia.org/wiki/5.1_surround_sound#Music)
    pub fn surround(
        front_left: i16,
        front_right: i16,
        front_center: i16,
        lfe: i16,
        surround_left: i16,
        surround_right: i16,
    ) -> AudioSample {
        AudioSample {
            front_left,
            front_right,
            center: front_center,
            lfe,
            surround_left,
            surround_right,
        }
    }
}
