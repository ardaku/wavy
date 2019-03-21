const MUSIC: &[u8] = include_bytes!("306.raw");

fn main() {
    let mut speaker = wavy::alsa2::Speaker::new(wavy::SampleRate::Normal);
    let mut cursor = 0;

    loop {
        speaker.play(&mut || {
            let sample_a = MUSIC[cursor];
            let sample_b = MUSIC[cursor + 1];
            let sample_c = MUSIC[cursor + 2];
            let sample_d = MUSIC[cursor + 3];

            let lsample = ((sample_a as u16)) | ((sample_b as u16) << 8);
            let rsample = ((sample_c as u16)) | ((sample_d as u16) << 8);

            cursor += 4;



            unsafe { (/*std::mem::transmute(lsample)*/0, std::mem::transmute(rsample)) }
        });
    }

/*    let mut audio_io = wavy::AudioIO::new(wavy::SampleRate::Normal);
    let mut cursor = 0;
    let mut alt = false;

    let mut x = 0;

    loop {
        audio_io.play(&mut || {
            /*let sample = x;
            if sample == std::i16::MAX - 255 {
                x = 0;
            }
            x += 256;
            sample*/

            let sample_a = MUSIC[cursor];
            let sample_b = MUSIC[cursor + 1];

            let sample = ((sample_a as u16)) | ((sample_b as u16) << 8);

//            if alt {
//                cursor += 4;
//            }
//            alt = !alt;

            cursor += 2;

            unsafe { std::mem::transmute(sample) }
        });
    }*/
}
