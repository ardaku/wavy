//! List `Speakers` and `Microphone` devices as they are connected.

use pasts::Loop;
use std::task::Poll::{self, Pending};
use wavy::{Connector, Microphone, Speakers};

type Exit = ();

struct State {
    speakers_connector: Connector<Speakers>,
    microphone_connector: Connector<Microphone>,
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
        speakers_connector: Connector::new(),
        microphone_connector: Connector::new(),
    };

    Loop::new(&mut state)
        .when(|s| &mut s.speakers_connector, State::speaker)
        .when(|s| &mut s.microphone_connector, State::microphone)
        .await;
}

fn main() {
    pasts::block_on(event_loop());
}
