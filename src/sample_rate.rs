/// 3 sample rates which are supported by this crate.
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum SampleRate {
    /// 24K sample rate.  This is good for where reducing latency matters more than quality.
    Sparse = 24_000_u32,
    /// 48K sample rate.  Use this for most things.
    Normal = 48_000_u32,
    /// 96K sample rate.  This is what is recorded in a studio (always downsampled to 48K for
    /// releases though).  Good for when you are slowing down parts of the audio later.
    Studio = 96_000_u32,
}
