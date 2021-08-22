//! Listen for when `Speakers` and `Microphones` are connected.

use pasts::Loop;
use std::task::Poll::{self, Pending};
use wavy::{Listener, Microphone, Speakers};

type Exit = ();

struct State {
    speakers_listener: Listener<Speakers>,
    microphone_listener: Listener<Microphone>,
}

impl State {
    fn speaker(&mut self, speaker: Speakers) -> Poll<Exit> {
        println!("New Speakers Connected: {}", speaker);
        Pending
    }

    fn microphone(&mut self, speaker: Microphone) -> Poll<Exit> {
        println!("New Microphone Connected: {}", speaker);
        Pending
    }
}

async fn event_loop() {
    let mut state = State {
        speakers_listener: Listener::new(),
        microphone_listener: Listener::new(),
    };

    Loop::new(&mut state)
        .when(|s| &mut s.speakers_listener, State::speaker)
        .when(|s| &mut s.microphone_listener, State::microphone)
        .await;
}

fn main() {
    pasts::block_on(event_loop());
}
