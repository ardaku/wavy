// Wavy
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

#![allow(unsafe_code)]

use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use std::task::Waker;

use crate::consts::SAMPLE_RATE;

#[repr(C)]
struct AudioQueueBuffer {
    audio_data_bytes_capacity: u32, // const
    audio_data: *mut i16,
    audio_data_byte_size: u32,
    user_data: *mut c_void,

    packet_description_capacity: u32, // const
    packet_descriptions: *const c_void,
    packet_description_count: u32,
}

#[repr(C)]
pub(crate) struct AudioStreamBasicDescription {
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

impl AudioStreamBasicDescription {
    pub(crate) const fn new(channels: u32) -> Self {
        Self {
            // Use wavy's target sample rate
            sample_rate: SAMPLE_RATE as f64,
            // Don't do any compression, use linear PCM (raw samples).
            format_id: u32::from_ne_bytes(*b"lpcm"),
            // Set flag for float (if unset, integer).
            format_flags: 1,
            // sizeof(f32) * channels
            bytes_per_packet: 4 * channels,
            // Always one
            frames_per_packet: 1,
            // Same as bytes_per_packet
            bytes_per_frame: 4 * channels,
            // Stereo (channels)
            channels_per_frame: channels,
            // sizeof(f32) * bits_in_a_byte
            bits_per_channel: 4 * 8,
            // Always zero
            reserved: 0,
        }
    }
}

// MacOS RunLoop is thread-safe.
#[repr(transparent)]
#[derive(Copy, Clone)]
struct RunLoop(*mut c_void);

unsafe impl Send for RunLoop {}
unsafe impl Sync for RunLoop {}

type OSStatus = i32;

extern "C" {
    fn CFRunLoopGetCurrent() -> RunLoop;
    fn CFRunLoopRun();
    fn AudioQueueNewOutput(
        in_format: *const AudioStreamBasicDescription,
        in_callback_proc: extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut AudioQueueBuffer,
        ),
        in_user_data: *mut c_void,
        in_callback_run_loop: RunLoop,
        in_callback_run_loop_mode: *const c_void,
        in_flags: u32,
        out_aq: *mut *mut c_void,
    ) -> OSStatus;
    fn AudioQueueNewInput(
        in_format: *const AudioStreamBasicDescription,
        in_callback_proc: extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut AudioQueueBuffer,
            *const c_void,
            u32,
            *const c_void,
        ),
        in_user_data: *mut c_void,
        in_callback_run_loop: RunLoop,
        in_callback_run_loop_mode: *const c_void,
        in_flags: u32,
        out_aq: *mut *mut c_void,
    ) -> OSStatus;
    fn AudioQueueEnqueueBuffer(
        in_audio_queue: *mut c_void,
        in_buffer: *mut AudioQueueBuffer,
        in_num_packet_descs: u32,
        in_packet_descs: *const c_void,
    ) -> OSStatus;
    fn AudioQueueStart(
        in_audio_queue: *mut c_void,
        in_start_timestamp: *const c_void,
    ) -> OSStatus;
    fn AudioQueueAllocateBuffer(
        in_audio_queue: *mut c_void,
        in_buffer_byte_size: u32,
        out_buffer: *mut *mut AudioQueueBuffer,
    ) -> OSStatus;
}

// Thread-local run loop for audio device (global state).
thread_local!(static RUN_LOOP: RunLoop = initialize());

/// Global initialization, may be called more than once.
fn initialize() -> RunLoop {
    let pair = std::sync::Arc::new((
        std::sync::Mutex::new(None),
        std::sync::Condvar::new(),
    ));
    let pair2 = Arc::clone(&pair);
    std::thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        // Get the run loop associated with this thread.
        let run_loop = unsafe { CFRunLoopGetCurrent() };
        *started = Some(run_loop);
        cvar.notify_one();
        // Block this thread with the run loop until the process exits.
        unsafe { CFRunLoopRun() }
    });

    // Wait to get the thread's Run Loop handle.
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    let run_loop = loop {
        if let Some(run_loop) = started.take() {
            break run_loop;
        }
        started = cvar.wait(started).unwrap();
    };
    run_loop
}

// User data associated with the speaker.
struct SpeakerContext {
    // Buffer for the next period * channels.
    buffer: Mutex<(Vec<f32>, Option<Waker>)>,
}

// AudioQueue callback for speaker.
extern "C" fn speaker_callback(
    user_data: *mut c_void,
    audio_queue: *mut c_void,
    audio_buffer: *mut AudioQueueBuffer,
) {
    // Cast user data.
    let user_data: *mut SpeakerContext = user_data.cast();

    // Write into buffer.
    let buffer: *mut f32 = unsafe { (*audio_buffer).audio_data.cast() };
    // Lock the mutex
    unsafe {
        let mut locked = (*user_data).buffer.lock().unwrap();
        // Copy the data
        let inbuf = &mut (*locked).0;
        for i in 0..((*audio_buffer).audio_data_byte_size / 4) {
            *buffer.offset(i as isize) =
                inbuf.get(i as usize).cloned().unwrap_or(0.0);
        }
        // Clear the written samples
        inbuf.clear();
        // Wake the main thread, telling it produce samples while the current ones are processing.
        if let Some(waker) = locked.1.take() {
            waker.wake();
        };
    }

    // Enqueue buffer to audio queue
    let status = unsafe {
        AudioQueueEnqueueBuffer(audio_queue, audio_buffer, 0, std::ptr::null())
    };
    if status != 0 {
        panic!("Failed enqueue {:?}", status);
    }
}

// AudioQueue callback for microphone.
extern "C" fn microphone_callback(
    user_data: *mut c_void,
    queue: *mut c_void,
    buffer: *mut c_void,
    start_timestamp: *const c_void,
    num_packet_descs: u32,
    packet_descs: *const c_void,
) {
}

/// A speaker (output) or microphone (input).
pub(crate) struct AudioQueue {
    audio_queue: *mut c_void,
    speaker_cx: *mut SpeakerContext,
}

impl Drop for AudioQueue {
    fn drop(&mut self) {
        // FIXME: Stop and Free the AudioQueue and speaker_cx.
        todo!()
    }
}

// If the audio format is not supported, returns b"fmt?" if the number of channels
// is not supported.
pub(crate) fn speaker(channels: u8) -> Result<AudioQueue, [u8; 4]> {
    RUN_LOOP.with(|run_loop| {
        let speaker_cx = Box::into_raw(Box::new(SpeakerContext {
            buffer: Mutex::new((Vec::new(), None)),
        }));

        let basic_desc = AudioStreamBasicDescription::new(channels.into());
        let mut audio_queue = std::mem::MaybeUninit::uninit();
        let status = unsafe {
            AudioQueueNewOutput(
                &basic_desc,
                speaker_callback,
                speaker_cx.cast(),
                *run_loop,
                std::ptr::null(),
                0,
                audio_queue.as_mut_ptr(),
            )
        };
        if status != 0 {
            return Err(status.to_ne_bytes());
        }
        let audio_queue = unsafe { audio_queue.assume_init() };

        // Create an empty buffer and enqueue the data.
        let mut audio_buffer = std::mem::MaybeUninit::uninit();
        let buf_byte_count =
            basic_desc.bytes_per_frame * crate::consts::PERIOD as u32;
        let status = unsafe {
            AudioQueueAllocateBuffer(
                audio_queue,
                buf_byte_count,
                audio_buffer.as_mut_ptr(),
            )
        };
        if status != 0 {
            panic!("Out of memory {}!", status);
        }
        let audio_buffer = unsafe { audio_buffer.assume_init() };
        // Set buffer length.
        unsafe {
            (*audio_buffer).audio_data_byte_size = buf_byte_count;
        }
        // Initialize to zero (silence).
        for i in 0..buf_byte_count {
            unsafe {
                let byte: *mut u8 = (*audio_buffer).audio_data.cast();
                *byte.offset(i as _) = 0;
            }
        }

        // Enqueue buffer to audio queue
        let status = unsafe {
            AudioQueueEnqueueBuffer(
                audio_queue,
                audio_buffer,
                0,
                std::ptr::null(),
            )
        };
        if status != 0 {
            return Err(status.to_ne_bytes());
        }

        // Now that we've created the audio queue an enqueued a buffer, we can start it.
        let status = unsafe { AudioQueueStart(audio_queue, std::ptr::null()) };
        if status != 0 {
            return Err(status.to_ne_bytes());
        }

        Ok(AudioQueue {
            audio_queue,
            speaker_cx,
        })
    })
}

/*
 *
void callback (void *ptr, AudioQueueRef queue, AudioQueueBufferRef buf_ref)
{
  OSStatus status;
  PhaseBlah *p = ptr;
  AudioQueueBuffer *buf = buf_ref;
  int nsamp = buf->mAudioDataByteSize / 2;
  short *samp = buf->mAudioData;
  int ii;
  printf ("Callback! nsamp: %d\n", nsamp);
  for (ii = 0; ii < nsamp; ii++) {
    samp[ii] = (int) (30000.0 * sin(p->phase));
    p->phase += p->phase_inc;
    //printf("phase: %.3f\n", p->phase);
  }
  p->count++;
  status = AudioQueueEnqueueBuffer (queue, buf_ref, 0, NULL);
  printf ("Enqueue status: %d\n", status);
}


int main (int argc, char *argv[])
{
  AudioQueueRef queue;
  PhaseBlah phase = { 0, 2 * 3.14159265359 * 450 / 44100 };
  OSStatus status;
  AudioStreamBasicDescription fmt = { 0 };
  AudioQueueBufferRef buf_ref, buf_ref2;

  fmt.mSampleRate = 44100;
  fmt.mFormatID = kAudioFormatLinearPCM;
  fmt.mFormatFlags = kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked;
  fmt.mFramesPerPacket = 1;
  fmt.mChannelsPerFrame = 1; // 2 for stereo
  fmt.mBytesPerPacket = fmt.mBytesPerFrame = 2; // x2 for stereo
  fmt.mBitsPerChannel = 16;

  status = AudioQueueNewOutput(&fmt, callback, &phase, CFRunLoopGetCurrent(),
                  kCFRunLoopCommonModes, 0, &queue);

  if (status == kAudioFormatUnsupportedDataFormatError) puts ("oops!");
  else printf("NewOutput status: %d\n", status);

    //

  status = AudioQueueAllocateBuffer (queue, 20000, &buf_ref);
  printf ("Allocate status: %d\n", status);

  AudioQueueBuffer *buf = buf_ref;
  printf ("buf: %p, data: %p, len: %d\n", buf, buf->mAudioData, buf->mAudioDataByteSize);
  buf->mAudioDataByteSize = 20000;

  callback (&phase, queue, buf_ref);
 */
