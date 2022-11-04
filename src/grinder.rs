use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::gui::MainWindowInput;
use crate::{Config, SImage};
use image::Rgba;
use imageproc::{drawing::Canvas, point::Point};

use rand::seq::SliceRandom;
use rand::Rng;

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

pub struct RateMeter {
    limit: usize,
    samples: Vec<usize>,
}

impl RateMeter {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            samples: vec![],
        }
    }

    pub fn sample(&mut self, value: usize) {
        self.samples.push(value);

        if self.samples.len() > self.limit {
            self.samples.remove(0);
        }
    }

    pub fn rate(&self) -> Option<f32> {
        if self.samples.len() >= self.limit {
            let sum: usize = self.samples.iter().sum();
            let rate = (sum as f32) / (self.limit as f32);
            Some(rate)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.samples = vec![];
    }

    /// Defaults false if there aren't enough samples
    pub fn is_below(&self, rate: f32) -> bool {
        if let Some(current_rate) = self.rate() {
            current_rate < rate
        } else {
            false
        }
    }
}

pub struct Grinder {
    reference: SImage,
    current: SImage,
    config: Config,
    tx: Sender<MainWindowInput>,
}

impl Grinder {
    pub fn new(tx: Sender<MainWindowInput>, config: Config) -> Self {
        let reference = SImage::open(&config.input).unwrap();
        let width = reference.width();
        let height = reference.height();

        Self {
            reference,
            current: SImage::new(width, height),
            config,
            tx,
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

        // tracks total iterations through the grinder loop
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

        // tracks when the last update message was sent from the grinder
        let mut last_update = Instant::now();

        loop {
            total_iterations += 1;
            attempts_at_radius += 1;

            // if we haven't sent an update in X milliseconds, send one.
            if (Instant::now() - last_update) > Duration::from_millis(250) {
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
                self.current.save(&self.config.output);
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

            // Don't copy the whole image; this is dumb. We should operate on a sample and then put it back into the image.
            // This is currently the most expensive part of this process!
            let mut candidate = self.current.clone();

            // let's draw a shape! Refactor me because this is kinda gross.
            match self.config.circles {
                true => imageproc::drawing::draw_filled_circle_mut(
                    &mut candidate.img,
                    (center_x as i32, center_y as i32),
                    current_radius as i32,
                    reference_color,
                ),
                false => {
                    let t1 = Triangle::from(&region);

                    let polypoints = t1.imageproc_points();

                    imageproc::drawing::draw_polygon_mut(
                        &mut candidate.img,
                        &polypoints,
                        reference_color,
                    );
                }
            };

            // check the deltas from that region
            let candidate_crop = candidate.section(&region);
            let candidate_delta = reference_crop.delta(&candidate_crop.img);
            let current_delta = reference_crop.delta(&current_crop.img);

            // if candidate is closer to the reference than the current best, promote it to current!
            if candidate_delta < current_delta {
                self.current = candidate;
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

pub enum RegionEdge {
    Top,
    Bottom,
    Right,
    Left,
}

pub const REGION_EDGES: [RegionEdge; 4] = [
    RegionEdge::Top,
    RegionEdge::Bottom,
    RegionEdge::Right,
    RegionEdge::Left,
];

#[derive(Default, Debug)]
pub struct Triangle {
    pub a: (i32, i32),
    pub b: (i32, i32),
    pub c: (i32, i32),
}

impl From<&Region> for Triangle {
    fn from(region: &Region) -> Self {
        // goal is to create a triangle that touches three random edges and overlaps the central point
        let mut triangle = Triangle::default();

        let mut rng = rand::thread_rng();
        for (idx, edge) in REGION_EDGES.choose_multiple(&mut rng, 3).enumerate() {
            let point = match edge {
                // Use minimums + 1 in order to avoid coordinate collisions with minimums
                RegionEdge::Top => (
                    rng.gen_range((region.min_x + 1)..region.max_x),
                    region.min_y,
                ),
                RegionEdge::Bottom => (
                    rng.gen_range((region.min_x + 1)..region.max_x),
                    region.max_y,
                ),
                RegionEdge::Right => (
                    region.max_x,
                    rng.gen_range((region.min_y + 1)..region.max_y),
                ),
                RegionEdge::Left => (
                    region.min_x,
                    rng.gen_range((region.min_y + 1)..region.max_y),
                ),
            };

            match idx {
                0 => triangle.a = point,
                1 => triangle.b = point,
                2 => triangle.c = point,
                _ => panic!("whoa, triangles only have three points"),
            }
        }

        triangle
    }
}

impl Triangle {
    pub fn imageproc_points(&self) -> Vec<Point<i32>> {
        vec![
            Point::new(self.a.0, self.a.1),
            Point::new(self.b.0, self.b.1),
            Point::new(self.c.0, self.c.1),
        ]
    }
}

pub struct ColorPicker {}

impl ColorPicker {
    pub fn sample(image_set: &SImage, x: u32, y: u32) -> Rgba<u8> {
        image_set.img.get_pixel(x, y)
    }
}
