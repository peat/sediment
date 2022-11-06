use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::gui::MainWindowInput;
use crate::{rate_meter::RateMeter, BuildConfig, SImage};
use image::{GenericImage, Rgba};
use imageproc::drawing::Canvas;
use rand::Rng;
use serde::Serialize;

#[derive(Serialize)]
pub struct Circle {
    pub x: u32,
    pub y: u32,
    pub radius: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug)]
pub struct Stats {
    pub total_attempts: usize,
    pub total_successes: usize,
    pub total_skips: usize,
    pub radius_attempts: usize,
    pub radius_successes: usize,
    pub radius_success_rate: f32,
    pub radius: usize,
    pub delta: usize,
    pub elapsed: Duration,
}

pub struct Builder {
    reference: SImage,
    current: SImage,
    config: BuildConfig,
    tx: Sender<MainWindowInput>,
    circles: Vec<Circle>,
}

impl Builder {
    pub fn new(tx: Sender<MainWindowInput>, config: BuildConfig) -> Self {
        let reference = SImage::open(&config.input).unwrap();
        let width = reference.width();
        let height = reference.height();

        Self {
            reference,
            current: SImage::new(width, height),
            config,
            tx,
            circles: vec![],
        }
    }

    pub fn send_update(&mut self, stats: Stats) {
        self.tx
            .send(MainWindowInput::Preview(self.current.img.clone()))
            .unwrap();

        self.tx.send(MainWindowInput::Stats(stats)).unwrap();
    }

    pub fn run(&mut self) {
        let start_time = Instant::now();

        // generates points to examine for shape placement
        let point_selector = PointSelector::new(&self.reference);

        // tracks total iterations through the builder loop
        let mut total_iterations: usize = 0;

        // total number of successful shape placements!
        let mut total_successes: usize = 0;

        // total number of skipped iterations, due to the selected region not being worth changing
        let mut total_skips: usize = 0;

        // The current radius of the shape we're attempting to place
        let mut current_radius = self.config.max_radius;

        // number of attempts at updating at the given radius size
        let mut attempts_at_radius: usize = 0;

        // number of successful attempts at updating at the given radius size
        let mut successes_at_radius: usize = 0;

        // tracks the success rate for the current radius
        let mut radius_success_rate = RateMeter::new(100);

        // tracks when the last update message was sent from the builder
        let mut last_update = Instant::now();

        loop {
            total_iterations += 1;
            attempts_at_radius += 1;

            // if we haven't sent an update in X milliseconds, send one.
            if (Instant::now() - last_update) > Duration::from_millis(20) {
                let stats = Stats {
                    total_attempts: total_iterations,
                    total_successes,
                    total_skips,
                    radius: current_radius as usize,
                    radius_attempts: attempts_at_radius,
                    radius_successes: successes_at_radius,
                    radius_success_rate: radius_success_rate.rate().unwrap_or_default(),
                    delta: self.reference.delta(&self.current.img),
                    elapsed: Instant::now() - start_time,
                };

                self.send_update(stats);
                last_update = Instant::now();
            }

            // examine the success rate to determine if we need to adjust our radius
            if radius_success_rate.is_below(self.config.radius_shrink_threshold)
                || (attempts_at_radius > self.config.radius_attempt_limit)
            {
                // reset our success rate calculator
                radius_success_rate.reset();

                // reset successes and attempts
                attempts_at_radius = 0;
                successes_at_radius = 0;

                // step down radius 10%, and guard against tiny values
                let raw_step = (current_radius as f32) * self.config.radius_step;
                let mut int_step = raw_step as u32;
                if raw_step < 1.0 {
                    int_step = 1;
                }

                current_radius -= int_step;
            }

            // if our radius hits the threshold we're done! Send the last update, write out the image, and return.
            if current_radius < self.config.min_radius {
                let stats = Stats {
                    total_attempts: total_iterations,
                    total_successes,
                    total_skips,
                    radius: current_radius as usize,
                    radius_attempts: attempts_at_radius,
                    radius_successes: successes_at_radius,
                    radius_success_rate: radius_success_rate.rate().unwrap_or_default(),
                    delta: self.reference.delta(&self.current.img),
                    elapsed: Instant::now() - start_time,
                };

                self.send_update(stats);
                self.current.save(&self.config.output);

                if let Some(raw_path) = &self.config.raw {
                    let mut writer = csv::Writer::from_path(raw_path).unwrap();
                    for c in self.circles.iter() {
                        writer.serialize(c).unwrap();
                    }
                }

                return;
            }

            // Picks the CENTER POINT of the region to be examined. This allows us to draw shapes that overlap
            // the edges of the image.
            let (center_x, center_y) = point_selector.point();

            let reference_color = ColorPicker::sample(&self.reference, center_x, center_y);
            let current_color = ColorPicker::sample(&self.current, center_x, center_y);

            // if reference pixel is the same as the current pixel, then skip ahead
            if reference_color == current_color {
                total_skips += 1;
                radius_success_rate.sample(0);
                continue;
            }

            let region = Region::new(center_x, center_y, current_radius);

            // get the delta between the reference and the current; if it's within a certain threshold, skip modifying it
            let reference_crop = self.reference.section(&region);
            let current_crop = self.current.section(&region);

            let reference_region_value = reference_crop.value();
            let current_region_value = current_crop.value();

            let region_similarity = (current_region_value as f32) / (reference_region_value as f32);

            // Skip ahead if this region is already looking really good.
            if (region_similarity < 1.0) && (region_similarity > self.config.similarity_threshold) {
                total_skips += 1;
                radius_success_rate.sample(0);
                continue;
            }

            // work from a crop of the current best image
            let mut candidate_crop = current_crop.clone();

            // let's draw a circle! Refactor me because this is kinda gross.
            imageproc::drawing::draw_filled_circle_mut(
                &mut candidate_crop.img,
                (candidate_crop.center_x, candidate_crop.center_y),
                current_radius as i32,
                reference_color,
            );

            // check the deltas from that region
            let candidate_delta = reference_crop.delta(&candidate_crop.img);
            let current_delta = reference_crop.delta(&current_crop.img);

            // if candidate is closer to the reference than the current best, promote it to current!
            if candidate_delta < current_delta {
                // copy the candidate crop into the current image
                self.current
                    .img
                    .copy_from(&candidate_crop.img, region.abs_x(), region.abs_y())
                    .unwrap();

                let circle = Circle {
                    x: center_x,
                    y: center_y,
                    radius: region.radius,
                    r: current_color.0[0],
                    g: current_color.0[1],
                    b: current_color.0[2],
                };

                self.circles.push(circle);

                radius_success_rate.sample(1);
                successes_at_radius += 1;
                total_successes += 1;
            } else {
                radius_success_rate.sample(0);
            }
        }
    }
}

struct PointSelector {
    width: u32,
    height: u32,
}

impl PointSelector {
    pub fn new(image_set: &SImage) -> Self {
        Self {
            width: image_set.width(),
            height: image_set.height(),
        }
    }

    pub fn point(&self) -> (u32, u32) {
        let mut rng = rand::thread_rng();

        let x = rng.gen_range(0..self.width);
        let y = rng.gen_range(0..self.height);

        (x, y)
    }
}

#[derive(Debug)]
pub struct Region {
    pub center_x: u32,
    pub center_y: u32,
    pub radius: u32,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl Region {
    pub fn new(center_x: u32, center_y: u32, radius: u32) -> Self {
        let i32x = center_x as i32;
        let i32y = center_y as i32;
        let i32radius = radius as i32;

        Self {
            center_x,
            center_y,
            radius,
            min_x: i32x - i32radius,
            min_y: i32y - i32radius,
            max_x: i32x + i32radius,
            max_y: i32y + i32radius,
        }
    }

    pub fn abs_x(&self) -> u32 {
        if self.min_x < 0 {
            0
        } else {
            self.min_x as u32
        }
    }

    pub fn abs_y(&self) -> u32 {
        if self.min_y < 0 {
            0
        } else {
            self.min_y as u32
        }
    }

    pub fn abs_width(&self) -> u32 {
        (self.max_x as u32) - self.abs_x()
    }

    pub fn abs_height(&self) -> u32 {
        (self.max_y as u32) - self.abs_y()
    }
}

pub struct ColorPicker {}

impl ColorPicker {
    pub fn sample(image_set: &SImage, x: u32, y: u32) -> Rgba<u8> {
        image_set.img.get_pixel(x, y)
    }
}