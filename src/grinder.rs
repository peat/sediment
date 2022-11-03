use std::sync::mpsc::Sender;

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
    pub radius: usize,
    pub delta: usize,
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

    pub fn run(&mut self) {
        let mut total_attempts: usize = 0;
        let mut total_successes: usize = 0;
        let mut total_skips: usize = 0;

        let mut attempts_at_radius: usize = 0;
        let mut successes_at_radius: usize = 0;
        loop {
            total_attempts += 1;

            if total_attempts % 100 == 0 {
                self.tx
                    .send(MainWindowInput::Preview(self.current.img.clone()))
                    .unwrap();

                self.tx
                    .send(MainWindowInput::Stats(Stats {
                        total_attempts,
                        total_successes,
                        total_skips,
                        radius: self.config.radius as usize,
                        radius_attempts: attempts_at_radius,
                        radius_successes: successes_at_radius,
                        delta: self.reference.delta(&self.current.img),
                    }))
                    .unwrap();
            }

            attempts_at_radius += 1;

            // make sure we have enough attempts at that size to start adjusting the radius
            if attempts_at_radius > 25 {
                let success_ratio = (successes_at_radius as f32) / (attempts_at_radius as f32);
                if (success_ratio < self.config.radius_success_threshold)
                    || (attempts_at_radius > self.config.radius_attempt_limit)
                {
                    // step down radius
                    self.config.radius -= 1;

                    // reset successes and attempts
                    attempts_at_radius = 0;
                    successes_at_radius = 0;

                    // if radius is 2, quit
                    if self.config.radius < 1 {
                        self.current.save(&self.config.output);
                        return;
                    }
                }
            }

            let (center_x, center_y) = PointSelector::new(&self.reference).point();

            let reference_color = ColorPicker::sample(&self.reference, center_x, center_y);
            let current_color = ColorPicker::sample(&self.current, center_x, center_y);

            // if reference pixel is the same as the current pixel, then continue onward
            if reference_color == current_color {
                total_skips += 1;
                // println!("{} Skipping: reference == current color", total_attempts);
                continue;
            }

            let region = Region::new(center_x, center_y, self.config.radius);

            // get the delta between the reference and the current; if it's within a certain threshold, skip modifying it
            let reference_crop = self.reference.crop(&region);
            let current_crop = self.current.crop(&region);
            let current_delta = reference_crop.delta(&current_crop.img);

            let skip_pixel_threshold = 5;
            let skip_region_threshold =
                (region.abs_height() as usize * region.abs_width() as usize) * skip_pixel_threshold;

            if current_delta < skip_region_threshold {
                total_skips += 1;
                // println!(
                //     "{} Skipping: delta {} < {}",
                //     total_attempts, current_delta, skip_region_threshold
                // );
                continue;
            }

            // let's draw a shape! This is the expensive bit, according to flame graphs
            let candidate = SImage {
                img: match self.config.circles {
                    true => imageproc::drawing::draw_filled_circle(
                        &self.current.img,
                        (center_x as i32, center_y as i32),
                        self.config.radius as i32,
                        reference_color,
                    )
                    .into(),
                    false => {
                        let t1 = Triangle::from(&region);

                        let polypoints = t1.imageproc_points();

                        imageproc::drawing::draw_polygon(
                            &self.current.img,
                            &polypoints,
                            reference_color,
                        )
                        .into()
                    }
                },
            };

            // check the deltas from that region
            let candidate_crop = candidate.crop(&region);
            let candidate_delta = reference_crop.delta(&candidate_crop.img);

            // if candidate is better than current, promote it to current!
            if candidate_delta < current_delta {
                self.current = candidate;
                successes_at_radius += 1;
                total_successes += 1;
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
