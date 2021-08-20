use wavy::{Microphone, Speakers};

fn main() {
    for speakers in Speakers::query() {
        println!("Found speaker: {}", speakers);
    }

    for microphone in Microphone::query() {
        println!("Found microphone: {}", microphone);
    }
}
