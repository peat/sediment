#![warn(clippy::all)]

mod builder;
mod canvas;
mod circle;
mod gui;
mod optimizer;
mod point_selector;
mod rate_meter;
mod region;
mod render;

pub use canvas::Canvas;
pub use circle::Circle;
pub use region::Region;

use builder::{Builder, BuilderUpdate, Stats};
use std::sync::mpsc::channel;
use std::thread;

use clap::{Args, Parser, Subcommand};

#[derive(Clone, Parser, Debug)]
pub struct Config {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Build a sediment image from an image file
    Build(BuildConfig),
    /// Render a sediment file to an image file
    Render(RenderConfig),
}

#[derive(Args, Clone, Debug)]
pub struct BuildConfig {
    /// Path to the input image file
    #[arg(short = 'i', long)]
    input: String,

    /// Path to the output image file (will overwrite)
    #[arg(short = 'o', long)]
    output: Option<String>,

    /// Path to the raw output file (will overwrite)
    #[arg(short = 'x', long)]
    raw: Option<String>,

    /// Maximum radius of the shapes to be placed
    #[arg(short = 'r', long, default_value_t = 500)]
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
    #[arg(short = 'a', long, default_value_t = 5000)]
    radius_attempt_limit: usize,

    /// Threshold for skipping shape placement
    #[arg(short = 's', long, short, default_value_t = 0.9)]
    similarity_threshold: f32,

    /// Display a GUI to view progress
    #[arg(short = 'g', long)]
    gui: bool,
}

#[derive(Args, Clone, Debug)]
pub struct RenderConfig {
    /// Path to the input .smt file
    #[arg(short = 'i', long)]
    input: String,

    /// Path to the output SVG file (will overwrite)
    #[arg(short = 's', long)]
    svg: Option<String>,

    /// Path to the output PNG file (will overwrite)
    #[arg(short = 'p', long)]
    png: Option<String>,
}

fn main() {
    let config = Config::parse();

    match config.command {
        Command::Build(build_config) => {
            print_build_config(&build_config);

            if build_config.gui {
                // UI run loop; doesn't exit.
                gui::run(build_config);
            } else {
                headless_run(build_config);
            }
        }

        Command::Render(render_config) => {
            crate::render::Render::new(render_config).run();
        }
    }
}

fn headless_run(config: BuildConfig) {
    let (builder_tx, builder_update_rx) = channel();

    thread::spawn(move || {
        let mut builder = Builder::new(builder_tx, config);
        builder.run();
    });

    loop {
        match builder_update_rx.recv() {
            Ok(BuilderUpdate::Stats(stats)) => print_stats(&stats),
            Ok(_) => continue,
            Err(_) => return,
        }
    }
}

fn print_build_config(config: &BuildConfig) {
    println!("{:#?}", config);
}

fn print_stats(stats: &Stats) {
    println!(
        "{}/{} {}% - {}s - Radius: {} ({}/{} {}%)",
        stats.total_successes,
        stats.total_attempts,
        (100.0 * ((stats.total_successes as f32) / (stats.total_attempts as f32))) as u32,
        stats.elapsed.as_secs(),
        stats.radius,
        stats.radius_successes,
        stats.radius_attempts,
        (100.0 * ((stats.radius_successes as f32) / (stats.radius_attempts as f32))) as u32,
    );
}
