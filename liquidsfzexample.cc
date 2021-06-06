// Compile this example with:
// $ g++ -o example example.cc `pkg-config --libs --cflags liquidsfz`
 
#include <liquidsfz.hh>
 
int
main (int argc, char **argv)
{
  if (argc != 2)
    {
      printf ("usage: %s <sfz_filename>\n", argv[0]);
      return 1;
    }
 
  LiquidSFZ::Synth synth;
  synth.set_sample_rate (48000);
 
  if (!synth.load (argv[1])) // load sfz
    {
      printf ("%s: failed to load sfz file '%s'\n", argv[0], argv[1]);
      return 1;
    }
 
  // start a note
  synth.add_event_note_on (0, 0, 60, 100);
 
  // render one block of audio data
  float left_output[1024];
  float right_output[1024];
  float *output[2] = { left_output, right_output };
  synth.process (output, 1024);
 
  // at this point we would typically play back the audio data
  for (int i = 0; i < 1024; i++)
    printf ("%d %f %f\n", i, left_output[i], right_output[i]);
}
