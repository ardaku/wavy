/* Ben's Audio Example for OSX 10.5+ (yeah Audio Queue)
     Ben White, Nov, 2011 

Makefile:


example: example.c
        gcc -o $@ $< -Wimplicit -framework AudioToolbox \
                -framework CoreFoundation -lm

*/

#include "AudioToolbox/AudioToolbox.h"

typedef struct {
  double phase, phase_inc;
  int count;
} PhaseBlah;


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

  status = AudioQueueAllocateBuffer (queue, 20000, &buf_ref);
  printf ("Allocate status: %d\n", status);

  AudioQueueBuffer *buf = buf_ref;
  printf ("buf: %p, data: %p, len: %d\n", buf, buf->mAudioData, buf->mAudioDataByteSize);
  buf->mAudioDataByteSize = 20000;

  callback (&phase, queue, buf_ref);

  status = AudioQueueAllocateBuffer (queue, 20000, &buf_ref2);
  printf ("Allocate2 status: %d\n", status);

  buf = buf_ref2;
  buf->mAudioDataByteSize = 20000;

  callback (&phase, queue, buf_ref2);

  status = AudioQueueSetParameter (queue, kAudioQueueParam_Volume, 1.0);
  printf ("Volume status: %d\n", status);

  status = AudioQueueStart (queue, NULL);
  printf ("Start status: %d\n", status);

  while (phase.count < 15)
    CFRunLoopRunInMode (
        kCFRunLoopDefaultMode,
        0.25, // seconds
        false // don't return after source handled
    );

  return 0;
}
