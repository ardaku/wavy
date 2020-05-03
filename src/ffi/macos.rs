use crate::frame::Frame;
use std::marker::PhantomData;
use std::task::Context;
use std::task::Poll;

pub(crate) struct Player<F: Frame> {
    _phantom: PhantomData<F>,
}

impl<F: Frame> Player<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let _ = sr;

        None
    }

    pub(crate) fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        let _ = cx;
    
        Poll::Pending
    }

    pub(crate) fn play_last(&mut self, audio: &[F]) -> usize {
        let _ = audio;
    
        0 // 0 frames were written.
    }
}

pub(crate) struct Recorder<F: Frame> {
    _phantom: PhantomData<F>,
}

impl<F: Frame> Recorder<F> {
    pub(crate) fn new(sr: u32) -> Option<Self> {
        let _phantom = PhantomData::<F>;
        let _ = sr;

        None
    }

    pub(crate) fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        let _ = cx;
    
        Poll::Pending
    }

    pub(crate) fn record_last(&mut self, audio: &mut Vec<F>) {
        let _ = audio;
    }
}

/*

use std::ffi::c_void;

use crate::*;

#[repr(C)]
struct AudioStreamBasicDescription {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

#[repr(C)]
struct AudioStreamPacketDescription {
    start_offset: i64,
    variable_frames_in_packet: u32,
    data_byte_size: u32,
}

#[repr(C)]
struct AudioQueueBuffer {
    audio_data_bytes_capacity: u32, // const
    audio_data: *mut i16,
    audio_data_byte_size: u32,
    user_data: *mut c_void,

    packet_description_capacity: u32, // const
    packet_descriptions: *const AudioStreamPacketDescription,
    packet_description_count: u32,
}

struct UserCtx<'a> {
    callback: Option<&'a mut dyn FnMut(&mut [[i16; 2]]) -> ()>,
}

unsafe extern "C" fn callback<'a>(
    user_ctx: *mut UserCtx<'a>,
    audio_queue: *mut AudioQueue,
    audio_buffer: *mut AudioQueueBuffer,
) {
    let slice: &mut [[i16; 2]] = std::slice::from_raw_parts_mut(
        (*audio_buffer).audio_data as *mut c_void as *mut [i16; 2],
        ((*audio_buffer).audio_data_byte_size / NUM_BYTES_PER_FRAME) as usize,
    );

    (*(*user_ctx).callback.as_mut().unwrap())(slice);

    let _status =
        AudioQueueEnqueueBuffer(audio_queue, audio_buffer, 0, std::ptr::null());
}

enum AudioQueue {}
enum CFRunLoop {}
enum CFString {}

type OSStatus = i32;

type AudioQueueOutputCallback = unsafe extern "C" fn(
    user_ctx: *mut UserCtx,
    audio_queue: *mut AudioQueue,
    audio_buffer: *mut AudioQueueBuffer,
);

#[link(name = "AudioToolbox", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFRunLoopCommonModes: *mut CFString;

    fn AudioQueueNewOutput(
        format: *const AudioStreamBasicDescription,
        callback_proc: AudioQueueOutputCallback,
        user_data: *mut c_void,
        callback_run_loop: *mut CFRunLoop,
        callback_run_loop_mode: *mut CFString,
        flags: i32,
        out: *mut *mut AudioQueue,
    ) -> OSStatus;

    fn CFRunLoopGetCurrent() -> *mut CFRunLoop;

    fn AudioQueueEnqueueBuffer(
        audio_queue: *mut AudioQueue,
        buffer: *mut AudioQueueBuffer,
        num_packets_descs: u32,
        packet_descs: *const c_void, // AudioStreamPacketDescription
    ) -> OSStatus;
}

const NUM_CHANNELS: u32 = 2;
const NUM_BYTES_PER_SAMPLE: u32 = 2;
const NUM_BYTES_PER_FRAME: u32 = NUM_CHANNELS * NUM_BYTES_PER_SAMPLE;

fn pcm_hw_params(sr: u32) -> AudioStreamBasicDescription {
    const AUDIO_FORMAT_LINEAR_PCM: [u8; 4] = *b"lpcm";

    const AUDIO_FORMAT_FLAG_IS_SIGNED_INT: u32 = 0x4;
    const AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 0x8;

    AudioStreamBasicDescription {
        sample_rate: sr as f64,
        format_id: unsafe {
            std::mem::transmute::<[u8; 4], u32>(AUDIO_FORMAT_LINEAR_PCM)
        },
        format_flags: AUDIO_FORMAT_FLAG_IS_SIGNED_INT
            | AUDIO_FORMAT_FLAG_IS_PACKED,
        bytes_per_packet: NUM_CHANNELS * NUM_BYTES_PER_SAMPLE,
        frames_per_packet: 1,
        bytes_per_frame: NUM_BYTES_PER_FRAME,
        channels_per_frame: NUM_CHANNELS,
        bits_per_channel: NUM_BYTES_PER_SAMPLE * 8,
        reserved: 0,
    }
}

pub struct Speaker {
    // sound_device: *mut snd_pcm_t,
    // buffer_size: u64,
    // period_size: u64,
    buffer: Vec<i16>,
}

impl Speaker {
    pub fn new(sr: u32) -> Result<Speaker, AudioError> {
        let hw_params = [pcm_hw_params(sr)];
        let mut audio_queue = std::mem::MaybeUninit::uninit();
        let mut user_ctx = UserCtx { callback: None };

        let _audio_queue_status = unsafe {
            AudioQueueNewOutput(
                hw_params.as_ptr(),
                callback,
                &mut user_ctx as *mut _ as *mut c_void,
                CFRunLoopGetCurrent(),
                kCFRunLoopCommonModes,
                0,
                audio_queue.as_mut_ptr(),
            )
        };

        let audio_queue = unsafe { audio_queue.assume_init() };

        /*let sound_device: *mut snd_pcm_t = pcm_open(false, b"default\0")?;
        let hw_params = pcm_hw_params(sr, sound_device)?;

        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        //        dbg!(buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(
                hw_params,
                &mut period_size,
                &mut d,
            );
        }
        //        dbg!(period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe { snd_pcm_prepare(sound_device) } < 0 {
            panic!(
                "Could not prepare!",
            );
        }

        let buffer_size = buffer_size as u64;
        let period_size = period_size as u64;*/

        // AudioQueueSetParameter (queue, kAudioQueueParam_Volume, 1.0);

        let buffer = Vec::new();

        Ok(Speaker {
            //sound_device,
            //buffer_size,
            //period_size,
            buffer,
        })
    }

    pub fn play(&mut self, generator: &mut dyn FnMut() -> (i16, i16)) {
        /*let _ = unsafe {
            snd_pcm_status(
                self.sound_device,
                self.status.as_mut_ptr() as *mut _,
            )
        };
        let avail = unsafe {
            snd_pcm_status_get_avail(self.status.as_ptr() as *const _)
        };
        let left = self.buffer_size - avail;

        let buffer_length = self.period_size * 4; // 16 bit (2 bytes) * Stereo (2 channels)

        let write = if left < buffer_length {
            buffer_length - left
        } else {
            0
        };

        self.buffer.clear();

        for _i in 0..write {
            let (l, r) = generator();
            self.buffer.push(l);
            self.buffer.push(r);
        }

        if unsafe {
            snd_pcm_writei(
                self.sound_device,
                self.buffer.as_mut_ptr(),
                write as u64,
            )
        } < 0
        {
            println!("Buffer underrun");
        }*/
    }
}

impl Drop for Speaker {
    fn drop(&mut self) {
        /*unsafe {
            snd_pcm_close(self.sound_device);
        }*/
    }
}

pub struct Microphone {
    // sound_device: *mut snd_pcm_t,
    buffer: Vec<i16>,
}

impl Microphone {
    pub fn new(sr: u32) -> Result<Microphone, AudioError> {
        /*        let sound_device: *mut snd_pcm_t = pcm_open(true, b"default\0")?;
        let hw_params = pcm_hw_params(sr, sound_device)?;

        // Get the buffer size.
        let mut buffer_size = 0;
        unsafe {
            snd_pcm_hw_params_get_buffer_size(hw_params, &mut buffer_size);
        }
        //        dbg!(buffer_size);
        let (mut period_size, mut d) = (0, 0);
        unsafe {
            snd_pcm_hw_params_get_period_size(
                hw_params,
                &mut period_size,
                &mut d,
            );
        }
        //        dbg!(period_size);

        unsafe {
            snd_pcm_hw_params_free(hw_params);
        }

        if unsafe { snd_pcm_start(sound_device) } < 0 {
            panic!(
                "Could not start!",
            );
        }*/

        let buffer = Vec::new();

        Ok(Microphone {
            // sound_device,
            buffer,
        })
    }

    pub fn record(&mut self, generator: &mut dyn FnMut(usize, i16, i16)) {
        /*let _ = unsafe {
            snd_pcm_status(
                self.sound_device,
                self.status.as_mut_ptr() as *mut _,
            )
        };
        let avail = unsafe {
            snd_pcm_status_get_avail(self.status.as_ptr() as *const _)
        };

        self.buffer.resize(avail as usize * 2, 0);

        if unsafe {
            snd_pcm_readi(
                self.sound_device,
                self.buffer.as_mut_ptr(),
                avail as u64,
            )
        } < 0
        {
            println!("Buffer overflow.");
        }

        for i in 0..((avail as usize) / 2) {
            let l = self.buffer[i * 2 as usize];
            let r = self.buffer[i * 2 + 1 as usize];

            generator(0, l, r);
        }*/
    }
}

impl Drop for Microphone {
    fn drop(&mut self) {
        /*unsafe {
            snd_pcm_close(self.sound_device);
        }*/
    }
}

*/
