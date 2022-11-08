use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::{point_selector::RandomPointSelector, rate_meter::RateMeter, BuildConfig, Canvas};
use image::{GenericImage, Rgba};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Circle {
    pub x: u32,
    pub y: u32,
    pub radius: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Circle {
    pub fn new(x: u32, y: u32, radius: u32, color: Rgba<u8>) -> Self {
        let r = color.0[0];
        let g = color.0[1];
        let b = color.0[2];

        Self {
            x,
            y,
            radius,
            r,
            g,
            b,
        }
    }
}

pub enum BuilderUpdate {
    Preview(image::DynamicImage),
    Stats(Stats),
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Stats {
    // tracks total iterations through the builder loop
    pub total_attempts: usize,
    // total number of successful shape placements!
    pub total_successes: usize,
    // total number of skipped iterations, due to the selected region not being worth changing
    pub total_skips: usize,
    // number of attempts at updating at the given radius size
    pub radius_attempts: usize,
    // number of successful attempts at updating at the given radius size
    pub radius_successes: usize,
    pub radius_success_rate: f32,
    // The current radius of the shape we're attempting to place
    pub radius: u32,
    pub delta: usize,
    pub elapsed: Duration,
}

pub struct Builder {
    reference: Canvas,
    current: Canvas,
    config: BuildConfig,
    tx: Sender<BuilderUpdate>,
    circles: Vec<Circle>,
    stats: Stats,
}

impl Builder {
    pub fn new(tx: Sender<BuilderUpdate>, config: BuildConfig) -> Self {
        let reference = Canvas::open(&config.input).unwrap();
        let width = reference.width();
        let height = reference.height();

        Self {
            reference,
            current: Canvas::new(width, height),
            config,
            tx,
            circles: vec![],
            stats: Stats::default(),
        }
    }

    pub fn send_update(&mut self, stats: Stats) {
        self.tx
            .send(BuilderUpdate::Preview(self.current.img.clone()))
            .unwrap();

        self.tx.send(BuilderUpdate::Stats(stats)).unwrap();
    }

    pub fn run(&mut self) {
        let start_time = Instant::now();

        // generates points to examine for shape placement
        let mut point_selector = RandomPointSelector::new(&self.reference);

        // tracks the success rate for the current radius
        let mut radius_success_rate = RateMeter::new(50);

        // tracks when the last update message was sent from the builder
        let mut last_update = Instant::now();

        // start with our max radius, woo!
        self.stats.radius = self.config.max_radius;

        loop {
            self.stats.total_attempts += 1;
            self.stats.radius_attempts += 1;

            // if we haven't sent an update in X milliseconds, send one.
            if (Instant::now() - last_update) > Duration::from_millis(20) {
                self.stats.radius_success_rate = radius_success_rate.rate().unwrap_or_default();
                self.stats.delta = self.reference.delta(&self.current.img);
                self.stats.elapsed = Instant::now() - start_time;

                self.send_update(self.stats);
                last_update = Instant::now();
            }

            // examine the success rate to determine if we need to adjust our radius
            if radius_success_rate.is_below(self.config.radius_shrink_threshold)
                || (self.stats.radius_attempts > self.config.radius_attempt_limit)
            {
                // reset our success rate calculator
                radius_success_rate.reset();

                // reset successes and attempts
                self.stats.radius_attempts = 0;
                self.stats.radius_successes = 0;

                // step down radius 10%, and guard against tiny values
                let raw_step = (self.stats.radius as f32) * self.config.radius_step;
                let mut int_step = raw_step as u32;
                if raw_step < 1.0 {
                    int_step = 1;
                }

                self.stats.radius -= int_step;
            }

            // if our radius hits the threshold we're done! Send the last update, write out the image, and return.
            if self.stats.radius < self.config.min_radius {
                self.stats.radius_success_rate = radius_success_rate.rate().unwrap_or_default();
                self.stats.delta = self.reference.delta(&self.current.img);
                self.stats.elapsed = Instant::now() - start_time;
                self.send_update(self.stats);

                if let Some(img_path) = &self.config.output {
                    self.current.save(img_path);
                }

                if let Some(raw_path) = &self.config.raw {
                    let mut writer = csv::Writer::from_path(raw_path).unwrap();
                    for c in self.circles.iter() {
                        writer.serialize(c).unwrap();
                    }
                }

                return;
            }

            // Picks the CENTER POINT of the region to be examined. This allows us to draw shapes that overlap
            // the edges of the image. The random point selector always returns Some() so unwrap()
            // is safe here ... unlike everywhere else, haha.
            let (center_x, center_y) = point_selector.next().unwrap();

            let reference_color = ColorPicker::sample(&self.reference, center_x, center_y);
            let current_color = ColorPicker::sample(&self.current, center_x, center_y);

            // if reference pixel is the same as the current pixel, then skip ahead
            if reference_color == current_color {
                self.stats.total_skips += 1;
                radius_success_rate.sample(0);
                continue;
            }

            let region = Region::new(center_x, center_y, self.stats.radius);

            // get the delta between the reference and the current; if it's within a certain threshold, skip modifying it
            let reference_crop = self.reference.section(&region);
            let current_crop = self.current.section(&region);

            let reference_region_value = reference_crop.value();
            let current_region_value = current_crop.value();

            let region_similarity = (current_region_value as f32) / (reference_region_value as f32);

            // Skip ahead if this region is already looking really good.
            if (region_similarity < 1.0) && (region_similarity > self.config.similarity_threshold) {
                self.stats.total_skips += 1;
                radius_success_rate.sample(0);
                continue;
            }

            // work from a crop of the current best image
            let mut candidate_crop = current_crop.clone();

            // let's draw a circle!
            imageproc::drawing::draw_filled_circle_mut(
                &mut candidate_crop.img,
                (candidate_crop.center_x, candidate_crop.center_y),
                self.stats.radius as i32,
                reference_color,
            );

            // check the deltas from that region
            let candidate_delta = reference_crop.delta(&candidate_crop.img);
            let current_delta = reference_crop.delta(&current_crop.img);

            // if candidate is closer to the reference than the current best, promote it to current!
            if candidate_delta < current_delta {
                // copy the candidate crop into the current image; marginally faster than just redrawing on the image
                self.current
                    .img
                    .copy_from(
                        &candidate_crop.img,
                        region.real_origin_x(),
                        region.real_origin_y(),
                    )
                    .unwrap();

                // create and save the circle
                let circle = Circle::new(center_x, center_y, self.stats.radius, reference_color);
                self.circles.push(circle);

                radius_success_rate.sample(1);
                self.stats.radius_successes += 1;
                self.stats.total_successes += 1;
            } else {
                radius_success_rate.sample(0);
            }
        }
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

    pub fn real_origin_x(&self) -> u32 {
        if self.min_x < 0 {
            0
        } else {
            self.min_x as u32
        }
    }

    pub fn real_origin_y(&self) -> u32 {
        if self.min_y < 0 {
            0
        } else {
            self.min_y as u32
        }
    }

    pub fn real_width(&self) -> u32 {
        (self.max_x as u32) - self.real_origin_x()
    }

    pub fn real_height(&self) -> u32 {
        (self.max_y as u32) - self.real_origin_y()
    }
}

pub struct ColorPicker {}

impl ColorPicker {
    pub fn sample(image_set: &Canvas, x: u32, y: u32) -> Rgba<u8> {
        use imageproc::drawing::Canvas; // namespace collision for get_pixel
        image_set.img.get_pixel(x, y)
    }
}
