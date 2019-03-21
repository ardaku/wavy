use wavy::prelude::*;

const MUSIC: &[u8] = include_bytes!("306.raw");

fn main() -> Result<(), AudioError> {
    let mut speaker = SpeakerSystem::new(wavy::SampleRate::Normal)?;
    let mut cursor = 0;
    let mut running = true;

    while running {
        speaker.play(&mut || {
            // When the last sample has been written, quit.
            if cursor >= MUSIC.len() {
                running = false;
                return (0, 0);
            }

            let sample_a = MUSIC[cursor];
            let sample_b = MUSIC[cursor + 1];
            let sample_c = MUSIC[cursor + 2];
            let sample_d = MUSIC[cursor + 3];

            let lsample = ((sample_a as u16)) | ((sample_b as u16) << 8);
            let rsample = ((sample_c as u16)) | ((sample_d as u16) << 8);

            cursor += 4;

            unsafe { (std::mem::transmute(lsample), std::mem::transmute(rsample)) }
        });
    }

    Ok(())
}
