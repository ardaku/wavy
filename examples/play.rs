//! This example records audio and plays it back in real time as it's being
//! recorded.

use pasts::{Interrupt, ThreadInterrupt};
use wavy::{Player, S16LEx2};

/// Shared data between recorder and player.
struct Shared {
    /// Audio Player
    player: Player<S16LEx2>,
    /// Generator
    gen: Generator,
}

#[derive(Debug)]
struct Generator {
    counter: i8,
    buf: Vec<S16LEx2>,
}

impl Generator {
    fn generate(&mut self) {
        for _ in 0..(1024 - self.buf.len()) {
            self.counter = self.counter.wrapping_add(1);
            let sample = self.counter as i16 * 255;
            self.buf.push(S16LEx2::new(sample, sample));
        }
    }
}

/// Create a new monitor.
async fn monitor() {
    async fn play(shared: &mut Shared) {
        shared.gen.generate();
        (&mut shared.player).await;
        let n_frames = shared.player.play_last(&mut shared.gen.buf);
        shared.gen.buf.drain(..n_frames.min(shared.gen.buf.len()));
        println!("played {} frames", n_frames);
    }

    let gen = Generator {
        counter: -1,
        buf: Vec::with_capacity(1024),
    };
    println!("Opening player…");
    let player = Player::new(48_000).unwrap();
    let mut shared = Shared { gen, player };
    println!("Done, entering async loop…");
    loop {
        play(&mut shared).await;
    }
}

/// Start the async executor.
fn main() {
    ThreadInterrupt::block_on(monitor())
}
