use wavy::{MicrophoneId, SpeakersId};

fn main() {
    for speaker in SpeakersId::query() {
        let name = format!("{}", speaker);
        if let Some(mut speakers) = speaker.connect() {
            println!("Found speaker: {}", name);
            if speakers.avail::<fon::mono::Mono32>() {
                println!(" - Mono");
            }
            if speakers.avail::<fon::stereo::Stereo32>() {
                println!(" - Stereo");
            }
            if speakers.avail::<fon::surround::Surround32>() {
                println!(" - Surround");
            }
        }
    }
    for microphone in MicrophoneId::query() {
        let name = format!("{}", microphone);
        if let Some(mut mic) = microphone.connect() {
            println!("Found microphone: {}", name);
            if mic.avail::<fon::mono::Mono32>() {
                println!(" - Mono");
            }
            if mic.avail::<fon::stereo::Stereo32>() {
                println!(" - Stereo");
            }
            if mic.avail::<fon::surround::Surround32>() {
                println!(" - Surround");
            }
        }
    }
}
