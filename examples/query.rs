use wavy::{MicrophoneId, SpeakersId};

fn main() {
    for speaker in SpeakersId::query() {
        if let Some(mut speakers) = speaker.connect() {
            println!("Found speaker: {}", speaker);
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
        if let Some(mut mic) = microphone.connect() {
            println!("Found microphone: {}", microphone);
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
