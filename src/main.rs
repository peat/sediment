use std::thread;

// use std::time::Instant;

mod grinder;
mod gui;
mod simage;

use grinder::Grinder;
use simage::SImage;
use std::sync::mpsc::channel;

use clap::Parser;

#[derive(Clone, Parser, Debug)]
pub struct Config {
    /// Path to the input image file
    #[arg(short, long)]
    input: String,

    /// Path to the output image file (will overwrite)
    #[arg(short, long)]
    output: String,

    /// Display a GUI to view progress
    #[arg(short, long)]
    gui: bool,

    /// Use circles instead of triangles
    #[arg(short, long)]
    circles: bool,

    /// Maximum radius of the shapes to be placed
    #[arg(short, long, default_value_t = 100)]
    radius: u32,

    /// Reduce the radius size when successes are lower than this rate
    #[arg(long, default_value_t = 0.2)]
    radius_success_threshold: f32,

    /// Reduce the radius size after this many attempts are made
    #[arg(long, default_value_t = 1000)]
    radius_attempt_limit: usize,
}

fn main() {
    let config = Config::parse();
    let grinder_config = config.clone();

    let (grinder_tx, main_window_rx) = channel();

    let handle = thread::spawn(move || {
        let mut grinder = Grinder::new(grinder_tx, grinder_config);
        grinder.run();
    });

    if config.gui {
        // UI run loop; doesn't exit.
        gui::run(main_window_rx);
    } else {
        loop {
            match main_window_rx.recv() {
                Ok(gui::MainWindowInput::Stats(stats)) => println!("{:?}", stats),
                Ok(_) => continue,
                Err(_) => return,
            }
        }
    }

    handle.join().unwrap();
}
