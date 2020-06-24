//! This example records audio and plays it back in real time as it's being
//! recorded.  Examples are in the public domain.

#![forbid(unsafe_code)]

#[macro_use]
extern crate devout;

use pasts::prelude::*;
use wavy::{Player, Recorder, S16LEx2};

use std::cell::RefCell;

const LOGGER: &str = "Monitor";

/// Shared data between recorder and player.
struct Shared {
    /// A stereo audio buffer.
    buffer: Vec<S16LEx2>,
}

/// Create a new monitor.
async fn monitor() {
    /// Extend buffer by slice of new frames from last plugged in device.
    async fn record(shared: &RefCell<Shared>) {
        let mut recorder = Recorder::<S16LEx2>::new(48_000).unwrap();
        loop {
            recorder.fut().await;
            let shared: &mut Shared = &mut *shared.borrow_mut();
            recorder.record_last(&mut shared.buffer);
        }
    }
    /// Drain double ended queue frames into last plugged in device.
    async fn play(shared: &RefCell<Shared>) {
        let mut player = Player::<S16LEx2>::new(48_000).unwrap();
        loop {
            player.fut().await;
            let shared: &mut Shared = &mut *shared.borrow_mut();
            let n_frames = player.play_last(shared.buffer.as_slice());
            shared.buffer.drain(..n_frames.min(shared.buffer.len()));
        }
    }

    let shared = RefCell::new(Shared { buffer: Vec::new() });
    let mut record = record(&shared);
    let mut play = play(&shared);
    [record.fut(), play.fut()].select().await;
    unreachable!()
}

fn main() {
    // Set panic handler for clean prints.
    cala_core::os::web::panic_hook();
    // Start the executor
    cala_core::os::web::block_on(monitor());
}
