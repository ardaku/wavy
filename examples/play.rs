//! This example records audio and plays it back in real time as it's being
//! recorded.

use pasts::{Interrupt, ThreadInterrupt};
use wavy::{Player, SampleRate, S16LEx2};

/// Shared data between recorder and player.
struct Shared {
    /// A boolean to indicate whether or not the program is still running.
    running: bool,
    /// Audio Player
    player: Player<S16LEx2>,
    /// Generator
    generator: Generator,
}

#[derive(Debug)]
pub struct Generator(i8);

impl Iterator for &mut Generator {
    type Item = S16LEx2;

    fn next(&mut self) -> Option<S16LEx2> {
        self.0 = self.0.wrapping_add(1);
        let sample = self.0 as i16 * 255;
        Some(S16LEx2::new(sample, sample))
    }
}

/// Create a new monitor.
async fn monitor() {
    /// Drain double ended queue frames into last plugged in device.
    async fn play(shared: &mut Shared) {
        let n_frames = shared.player.play_last(&mut shared.generator).await;
        println!("played {} frames", n_frames);
    }

    let running = true;
    let generator = Generator(-1);
    println!("Opening player…");
    let player = Player::new(SampleRate::Normal).unwrap();
    let mut shared = Shared {
        running,
        generator,
        player,
    };
    println!("Done, entering async loop…");
    pasts::tasks!(shared while shared.running; [play]);
}

/// Start the async executor.
fn main() {
    ThreadInterrupt::block_on(monitor())
}
