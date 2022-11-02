use std::thread;

// use std::time::Instant;

mod grinder;
mod gui;
mod simage;

use grinder::Grinder;
use simage::SImage;
use std::sync::mpsc::channel;

fn main() {
    let (grinder_tx, main_window_input_rx) = channel();

    let handle = thread::spawn(move || {
        let mut grinder = Grinder::new(grinder_tx, "teapot.jpg");
        grinder.run();
    });

    // UI run loop; doesn't exit.
    gui::run(main_window_input_rx);

    handle.join().unwrap();
}
