#include <alsa/asoundlib.h>
#include <iostream>
using namespace std;

// Globals are generally a bad idea in code.  We're using one here to keep it simple.
snd_pcm_t * _soundDevice;

bool Init(const char *name)
{
  int i;
  int err;
  snd_pcm_hw_params_t *hw_params;

  if( name == NULL )
  {
      // Try to open the default device
      err = snd_pcm_open( &_soundDevice, "plughw:0,0", SND_PCM_STREAM_PLAYBACK, 0 );
  }
  else
  {
      // Open the device we were told to open.
      err = snd_pcm_open (&_soundDevice, name, SND_PCM_STREAM_PLAYBACK, 0);
  }

  // Check for error on open.
  if( err < 0 )
  {
      cout << "Init: cannot open audio device " << name << " (" << snd_strerror (err) << ")" << endl;
      return false;
  }
  else
  {
      cout << "Audio device opened successfully." << endl;
  }

  // Allocate the hardware parameter structure.
  if ((err = snd_pcm_hw_params_malloc (&hw_params)) < 0)
  {
      cout << "Init: cannot allocate hardware parameter structure (" << snd_strerror (err) << ")" << endl;
      return false;
  }

  if ((err = snd_pcm_hw_params_any (_soundDevice, hw_params)) < 0)
  {
      cout << "Init: cannot initialize hardware parameter structure (" << snd_strerror (err) << ")" << endl;
      return false;
  }

  // Enable resampling.
  unsigned int resample = 1;
  err = snd_pcm_hw_params_set_rate_resample(_soundDevice, hw_params, resample);
  if (err < 0)
  {
      cout << "Init: Resampling setup failed for playback: " << snd_strerror(err) << endl;
      return err;
  }

  // Set access to RW interleaved.
  if ((err = snd_pcm_hw_params_set_access (_soundDevice, hw_params, SND_PCM_ACCESS_RW_INTERLEAVED)) < 0)
  {
      cout << "Init: cannot set access type (" << snd_strerror (err) << ")" << endl;
      return false;
  }

  if ((err = snd_pcm_hw_params_set_format (_soundDevice, hw_params, SND_PCM_FORMAT_S16_LE)) < 0)
  {
      cout << "Init: cannot set sample format (" << snd_strerror (err) << ")" << endl;
      return false;
  }

  // Set channels to stereo (2).
  if ((err = snd_pcm_hw_params_set_channels (_soundDevice, hw_params, 2)) < 0)
  {
      cout << "Init: cannot set channel count (" << snd_strerror (err) << ")" << endl;
      return false;
  }

  // Set sample rate.
  unsigned int actualRate = 48000;
  if ((err = snd_pcm_hw_params_set_rate_near (_soundDevice, hw_params, &actualRate, 0)) < 0)
  {
      cout << "Init: cannot set sample rate to 48000. (" << snd_strerror (err) << ")"  << endl;
      return false;
  }
  if( actualRate < 48000 )
  {
      cout << "Init: sample rate does not match requested rate. (" << "48000 requested, " << actualRate << " acquired)" << endl;
  }

  // Apply the hardware parameters that we've set.
  if ((err = snd_pcm_hw_params (_soundDevice, hw_params)) < 0)
  {
      cout << "Init: cannot set parameters (" << snd_strerror (err) << ")" << endl;
      return false;
  }
  else
  {
     cout << "Audio device parameters have been set successfully." << endl;
  }

  // Get the buffer size.
  snd_pcm_uframes_t bufferSize;
  snd_pcm_hw_params_get_buffer_size( hw_params, &bufferSize );
  // If we were going to do more with our sound device we would want to store
  // the buffer size so we know how much data we will need to fill it with.
  cout << "Init: Buffer size = " << bufferSize << " frames." << endl;

  // Display the bit size of samples.
  cout << "Init: Significant bits for linear samples = " << snd_pcm_hw_params_get_sbits(hw_params) << endl;

  // Free the hardware parameters now that we're done with them.
  snd_pcm_hw_params_free (hw_params);

  // Prepare interface for use.
  if ((err = snd_pcm_prepare (_soundDevice)) < 0)
  {
      cout << "Init: cannot prepare audio interface for use (" << snd_strerror (err) << ")" << endl;
      return false;
  }
  else
  {
      cout << "Audio device has been prepared for use." << endl;
  }

  short buf[128];

  for(int i = 0; i < 64; i++) {
	  buf[i] = i * 255 * 2;
	  buf[i + 64] = 65535 - buf[i];
  }

  for(int i = 0; i < 100; i++) {
  	snd_pcm_writei(_soundDevice, buf, 128);
  }

  while(true) { }

  return true;
}

bool UnInit()
{
  snd_pcm_close (_soundDevice);
  cout << "Audio device has been uninitialized." << endl;
  return true;
}

int main( int argv, char **argc )
{
        Init(NULL);
        UnInit();
        return 0;
}

