use std::thread;

// use std::time::Instant;

mod builder;
mod gui;
mod rate_meter;
mod simage;

use builder::Builder;
use simage::SImage;
use std::sync::mpsc::channel;

use clap::{Args, Parser, Subcommand};

#[derive(Clone, Parser, Debug)]
pub struct Config {
    #[command(subcommand)]
    command: Command,

    /// Display a GUI to view progress
    #[arg(short = 'g', long)]
    gui: bool,
}

#[derive(Args, Clone, Debug)]
pub struct BuildConfig {
    /// Path to the input image file
    #[arg(short = 'i', long)]
    input: String,

    /// Path to the output image file (will overwrite)
    #[arg(short = 'o', long)]
    output: String,

    /// Path to the raw output file (will overwrite)
    #[arg(short = 'x', long)]
    raw: Option<String>,

    /// Maximum radius of the shapes to be placed
    #[arg(short = 'r', long, default_value_t = 100)]
    max_radius: u32,

    /// Minimum radius of the shapes to be placed
    #[arg(short = 'm', long, default_value_t = 1)]
    min_radius: u32,

    /// Shrink the radius size when successes are lower than this rate
    #[arg(short = 't', long, default_value_t = 0.2)]
    radius_shrink_threshold: f32,

    /// Amount to shrink the radius at each step
    #[arg(short = 'p', long, default_value_t = 0.1)]
    radius_step: f32,

    /// Reduce the radius size after this many attempts are made
    #[arg(short = 'a', long, default_value_t = 1000)]
    radius_attempt_limit: usize,

    /// Threshold for skipping shape placement
    #[arg(short = 's', long, short, default_value_t = 0.9)]
    similarity_threshold: f32,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    Build(BuildConfig),
}

fn main() {
    let config = Config::parse();

    let (builder_tx, main_window_rx) = channel();

    match config.command {
        Command::Build(build_config) => {
            thread::spawn(move || {
                Builder::new(builder_tx, build_config).run();
            })
            .join()
            .unwrap();
        }
    }

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
}
