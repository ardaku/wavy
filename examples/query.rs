use wavy::{Microphone, Speakers};

fn main() {
    for speakers in Speakers::query() {
        println!("Found speaker: {}", speakers);

        if speakers.supports::<fon::mono::Mono32>() {
            println!(" - Mono");
        }
        if speakers.supports::<fon::stereo::Stereo32>() {
            println!(" - Stereo");
        }
        if speakers.supports::<fon::surround::Surround32>() {
            println!(" - Surround");
        }
    }

    for microphone in Microphone::query() {
        println!("Found microphone: {}", microphone);

        if microphone.supports::<fon::mono::Mono32>() {
            println!(" - Mono");
        }
        if microphone.supports::<fon::stereo::Stereo32>() {
            println!(" - Stereo");
        }
        if microphone.supports::<fon::surround::Surround32>() {
            println!(" - Surround");
        }
    }
}
