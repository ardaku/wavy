// This example records audio and plays it back in real time as it's being
// recorded.

use fon::{mono::Mono32, Audio, Sink};
use pasts::exec;
use wavy::{Microphone, MicrophoneStream, Speakers, SpeakersSink};

static AWOKEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

struct TheadedFuture(std::sync::Arc<std::sync::Mutex<Option<std::task::Waker>>>);

impl std::future::Future for &mut TheadedFuture {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let result = if AWOKEN.compare_and_swap(true, false, std::sync::atomic::Ordering::SeqCst) {
            std::task::Poll::Ready(())
        } else {
            // println!("Not ready yet, yielding");
            std::task::Poll::Pending
        };
        let mut a = self.0.lock().unwrap();
        *a = Some(cx.waker().clone());
        result
    }
}

fn thread(waker: std::sync::Arc<std::sync::Mutex<Option<std::task::Waker>>>) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if !AWOKEN.compare_and_swap(false, true, std::sync::atomic::Ordering::SeqCst) {
        } else {
        }
        // Ready
        if let Some(ref a) = *waker.lock().unwrap() {
            a.wake_by_ref();
        }
    }
}

/// An event handled by the event loop.
enum Event<'a> {
    /// Speaker is ready to play more audio.
    Play(SpeakersSink<'a, Mono32>),
    /// Microphone has recorded some audio.
    Record(MicrophoneStream<'a, Mono32>),
    Test(()),
}

/// Shared state between tasks on the thread.
struct State {
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Mono32>,
}

impl State {
    /// Event loop.
    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Play(mut speakers) => speakers.stream(self.buffer.drain()),
            Event::Record(microphone) => self.buffer.extend(microphone),
            _ => {}
        }
    }
}

/// Program start.
fn main() {
    let mut state = State { buffer: Audio::with_silence(48_000, 0) };
    let mut speakers = Speakers::default();
    let mut microphone = Microphone::default();

    let waker = std::sync::Arc::new(std::sync::Mutex::new(None));
    let wkrtw = waker.clone();

    std::thread::spawn(move || thread(wkrtw));

    let mut thread_fut = TheadedFuture(waker);

    exec!(state.event(pasts::wait! {
        Event::Play(speakers.play().await),
        Event::Record(microphone.record().await),
        // Event::Test((&mut thread_fut).await)
    }))
}
