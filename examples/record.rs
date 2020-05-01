//! This example records audio and plays it back in real time as it's being
//! recorded.

use pasts::ThreadInterrupt;
use pasts::prelude::*;
use wavy::{Player, Recorder, S16LEx2};

use std::cell::RefCell;

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
            println!("Record Waiting...");
            (&mut recorder).await;
            let shared: &mut Shared = &mut *shared.borrow_mut();
            println!("Initial Len: {}", shared.buffer.len());
            recorder.record_last(&mut shared.buffer);
            println!("Final len: {}", shared.buffer.len());
        }
    }
    /// Drain double ended queue frames into last plugged in device.
    async fn play(shared: &RefCell<Shared>) {
        let mut player = Player::<S16LEx2>::new(48_000).unwrap();
    
        loop {
            // println!("Player Waiting...");
            (&mut player).await;
            let shared: &mut Shared = &mut *shared.borrow_mut();
            let n_frames = player.play_last(shared.buffer.as_slice());
            shared.buffer.drain(..n_frames.min(shared.buffer.len()));
        }
    }

    let mut shared = Shared { buffer: Vec::new() };
    let mut shared = RefCell::new(shared);
    let mut record = record(&shared);
    let mut play = play(&shared);
    println!("Entering async loop…");
    [record.dyn_fut(), play.dyn_fut()].select().await;
    unreachable!()
}

/// Start the async executor.
fn main() {
    ThreadInterrupt::block_on(monitor())
}
