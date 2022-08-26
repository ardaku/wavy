use wavy::{Microphone, Speakers};

fn main() {
    for speakers in Speakers::<0>::query() {
        println!("Found speaker: {}", speakers);

        // FIXME
        /*if speakers.config::<fon::mono::Mono32>() {
            println!(" - Mono");
        }
        if speakers.config::<fon::stereo::Stereo32>() {
            println!(" - Stereo");
        }
        if speakers.config::<fon::surround::Surround32>() {
            println!(" - Surround");
        }*/
    }

    for microphone in Microphone::<0>::query() {
        println!("Found microphone: {}", microphone);

        // FIXME
        /*if microphone.supports::<fon::mono::Mono32>() {
            println!(" - Mono");
        }
        if microphone.supports::<fon::stereo::Stereo32>() {
            println!(" - Stereo");
        }
        if microphone.supports::<fon::surround::Surround32>() {
            println!(" - Surround");
        }*/
    }
}
