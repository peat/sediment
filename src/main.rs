use std::thread;

// use std::time::Instant;

mod grinder;
mod gui;
mod simage;

use grinder::Grinder;
use simage::SImage;
use std::sync::mpsc::channel;

fn main() {
    let (grinder_tx, main_window_rx) = channel();
    let (main_window_tx, grinder_rx) = channel();

    let handle = thread::spawn(move || {
        let mut grinder = Grinder::new(grinder_rx, grinder_tx, "landscape.jpg");
        grinder.run();
    });

    // UI run loop; doesn't exit.
    gui::run(main_window_rx, main_window_tx);

    handle.join().unwrap();
}
