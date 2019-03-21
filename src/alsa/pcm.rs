use super::error::*;
use super::{Direction, ValueOr};
use crate::alsa::alsa;
use crate::alsa::alsa::Context;
use libc::{c_int, c_uint, c_ulong, c_void};
use std::ffi::CString;
use std::ptr;

pub(crate) type Frames = alsa::snd_pcm_sframes_t;
pub(crate) struct PCM(*mut alsa::snd_pcm_t);

unsafe impl Send for PCM {}

impl PCM {
    /// Wrapper around open that takes a &str instead of a &CStr
    pub(crate) fn new(
        context: &Context,
        name: &str,
        dir: Direction, /*, nonblock: bool*/
    ) -> Result<PCM> {
        let name = CString::new(name).unwrap();
        let mut r = ptr::null_mut();
        let stream = match dir {
            Direction::Capture => 1,
            Direction::Playback => 0,
        };
/*        let flags = match dir {
            Direction::Capture => 2,
            Direction::Playback => 1,
        }; /*if nonblock { *//* } else { 0;*/*/
        acheck!(context, snd_pcm_open(&mut r, name.as_ptr(), stream, 0)).map(|_| PCM(r))
    }

    pub(crate) fn prepare(&self, context: &Context) {
        unsafe {
            (context.snd_pcm_prepare)(self.0);
        }
    }

    pub(crate) fn start(&self, context: &Context) {
        unsafe {
            (context.snd_pcm_start)(self.0);
        }
    }

    pub(crate) fn recover(&self, context: &Context, err: c_int, silent: bool) -> Result<()> {
        acheck!(
            context,
            snd_pcm_recover(self.0, err, if silent { 1 } else { 0 })
        )
        .map(|_| ())
    }

    pub(crate) fn status(&self, context: &Context) -> Result<Status> {
        let z = Status::new(context);
        acheck!(context, snd_pcm_status(self.0, z.ptr())).map(|_| z)
    }

    pub(crate) fn hw_params(&self, context: &Context, h: &HwParams) -> Result<()> {
        acheck!(context, snd_pcm_hw_params(self.0, h.0)).map(|_| ())
    }

    pub(crate) fn hw_params_current<'a>(&'a self, context: &Context) -> Result<HwParams<'a>> {
        HwParams::new(context, &self)
            .and_then(|h| acheck!(context, snd_pcm_hw_params_current(self.0, h.0)).map(|_| h))
    }

    pub(crate) fn drop(&self, context: &Context) {
        unsafe { (context.snd_pcm_close)(self.0) };
    }

    /// On success, returns number of *frames* written.
    /// (Multiply with number of channels to get number of items in buf successfully written.)
    pub(crate) fn writei(&self, context: &Context, buf: &[i16]) -> Result<usize> {
        let nsamples = buf.len() as c_ulong;

        acheck!(
            context,
            snd_pcm_writei(self.0, buf.as_ptr() as *const c_void, nsamples)
        )
        .map(|r| r as usize)
    }

    /// On success, returns number of *frames* read.
    /// (Multiply with number of channels to get number of items in buf successfully read.)
    pub(crate) fn readi(&self, context: &Context, buf: &mut [i16]) -> Result<usize> {
        let nsamples = buf.len() as c_ulong;

        acheck!(
            context,
            snd_pcm_readi(self.0, buf.as_mut_ptr() as *mut c_void, nsamples)
        )
        .map(|r| r as usize)
    }
}

#[allow(unused)]
#[repr(u32)]
pub(crate) enum Access {
    MMapInterleaved = 0,
    MMapNonInterleaved = 1,
    MMapComplex = 2,
    RWInterleaved = 3,
    RWNonInterleaved = 4,
}

/// [snd_pcm_hw_params_t](http://www.alsa-project.org/alsa-doc/alsa-lib/group___p_c_m___h_w___params.html) wrapper
pub(crate) struct HwParams<'a>(*mut alsa::snd_pcm_hw_params_t, &'a PCM);

impl<'a> HwParams<'a> {
    fn new(context: &Context, a: &'a PCM) -> Result<HwParams<'a>> {
        let mut p = ptr::null_mut();
        acheck!(context, snd_pcm_hw_params_malloc(&mut p)).map(|_| HwParams(p, a))
    }

    pub(crate) fn any(context: &Context, a: &'a PCM) -> Result<HwParams<'a>> {
        HwParams::new(context, a)
            .and_then(|p| acheck!(context, snd_pcm_hw_params_any(a.0, p.0)).map(|_| p))
    }

    pub(crate) fn set_channels(&self, context: &Context, v: u32) -> Result<()> {
        acheck!(
            context,
            snd_pcm_hw_params_set_channels((self.1).0, self.0, v as c_uint)
        )
        .map(|_| ())
    }

    pub(crate) fn get_channels(&self, context: &Context) -> Result<u32> {
        let mut v = 0;
        acheck!(context, snd_pcm_hw_params_get_channels(self.0, &mut v)).map(|_| v as u32)
    }

    pub(crate) fn set_rate(&self, context: &Context, v: u32, dir: ValueOr) -> Result<()> {
        acheck!(
            context,
            snd_pcm_hw_params_set_rate((self.1).0, self.0, v as c_uint, dir as c_int)
        )
        .map(|_| ())
    }

    pub(crate) fn get_rate(&self, context: &Context) -> Result<u32> {
        let (mut v, mut d) = (0, 0);
        acheck!(context, snd_pcm_hw_params_get_rate(self.0, &mut v, &mut d)).map(|_| v as u32)
    }

    pub(crate) fn set_format(&self, context: &Context, v: c_int) -> Result<()> {
        acheck!(context, snd_pcm_hw_params_set_format((self.1).0, self.0, v)).map(|_| ())
    }

    pub(crate) fn set_access(&self, context: &Context, v: Access) -> Result<()> {
        acheck!(
            context,
            snd_pcm_hw_params_set_access((self.1).0, self.0, v as c_uint)
        )
        .map(|_| ())
    }

    pub(crate) fn get_period_size(&self, context: &Context) -> Result<Frames> {
        let (mut v, mut d) = (0, 0);
        acheck!(
            context,
            snd_pcm_hw_params_get_period_size(self.0, &mut v, &mut d)
        )
        .map(|_| v as Frames)
    }

    pub(crate) fn get_buffer_size(&self, context: &Context) -> Result<usize> {
        let mut v: c_ulong = 0;
        acheck!(context, snd_pcm_hw_params_get_buffer_size(self.0, &mut v)).map(|_| v as usize)
    }

    pub(crate) fn drop(&self, context: &Context) {
        unsafe { (context.snd_pcm_hw_params_free)(self.0) };
    }
}

const STATUS_SIZE: usize = 152;

pub(crate) struct Status([u8; STATUS_SIZE]);

impl Status {
    fn new(context: &Context) -> Status {
        assert!(unsafe { (context.snd_pcm_status_sizeof)() } as usize <= STATUS_SIZE);
        Status([0; STATUS_SIZE])
    }

    fn ptr(&self) -> *mut alsa::snd_pcm_status_t {
        self.0.as_ptr() as *const _ as *mut alsa::snd_pcm_status_t
    }

    pub(crate) fn get_avail(&self, context: &Context) -> usize {
        unsafe { (context.snd_pcm_status_get_avail)(self.ptr()) as usize }
    }
}
