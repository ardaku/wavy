/// Replaces constants ending with PLAYBACK/CAPTURE as well as
/// INPUT/OUTPUT
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Direction {
    Playback,
    Capture,
}

/// Used to restrict hw parameters. In case the submitted
/// value is unavailable, in which direction should one search
/// for available values?
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ValueOr {
    /// The value set is the submitted value, or the nearest
    Nearest = 0,
}

mod error;

pub mod pcm;

mod alsa;
pub(crate) use self::alsa::Context;

// /// Standard Audio Hz for Opus.
// pub const HZ_48K: u32 = 48_000;

/*pub(crate) fn set_settings(context: &Context, pcm: &pcm::PCM, stereo: bool) {
    // Set hardware parameters: 48000 Hz / Mono / 16 bit
    let hwp = pcm::HwParams::any(context, pcm).unwrap();
    hwp.set_channels(context, if stereo { 2 } else { 1 })
        .unwrap();
    hwp.set_rate(context, HZ_48K, ValueOr::Nearest)
        .unwrap();
    let rate = hwp.get_rate(context).unwrap();
    assert_eq!(rate, HZ_48K);
    hwp.set_format(context, {
        if cfg!(target_endian = "little") {
            2
        } else if cfg!(target_endian = "big") {
            3
        } else {
            unreachable!()
        }
    })
    .unwrap();
    hwp.set_access(context, pcm::Access::RWInterleaved)
        .unwrap();
    pcm.hw_params(context, &hwp).unwrap();
    hwp.drop(context);
}*/
