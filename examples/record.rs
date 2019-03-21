use std::collections::VecDeque;

fn main() {
    let mut audio_io = wavy::AudioIO::new(wavy::SampleRate::Normal);
    let mut audio = VecDeque::new();

    loop {
        let mut buffer = [0i16; 512];
        let samples = audio_io.record(&mut buffer);
        audio.extend(&buffer[0..samples]);

        audio_io.play(&mut || {
            audio.pop_front().unwrap_or(0)
        });
    }
}
