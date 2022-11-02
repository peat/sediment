use rand::seq::SliceRandom;
use rand::Rng;

// use std::time::Instant;

mod main_window;
mod simage;

use simage::SImage;

use image::Rgba;
use imageproc::{drawing::Canvas, point::Point};

fn main() {
    let reference = SImage::open("teapot.jpg").unwrap();
    let mut current = SImage::new(reference.width(), reference.height());

    let mut radius = 500;
    let mut attempts_at_radius = 0f32;
    let mut successes_at_radius = 0f32;

    loop {
        attempts_at_radius += 1.0;
        let success_ratio = successes_at_radius / attempts_at_radius;
        println!(
            "{} of {} ({}) at {}px",
            successes_at_radius, attempts_at_radius, success_ratio, radius
        );

        if attempts_at_radius > 10.0 {
            if (success_ratio < 0.2) || (attempts_at_radius > 2000.0) {
                // step down radius
                radius -= 5;

                // reset successes and attempts
                attempts_at_radius = 0.0;
                successes_at_radius = 0.0;

                // if radius is 5, quit
                if radius < 5 {
                    current.save("teapot-derp.png");
                    return;
                }
            }
        }

        let ps = PointSelector::new(&reference);
        let (center_x, center_y) = ps.point();

        let region = Region::new(center_x, center_y, radius);
        let t1 = Triangle::from(&region);

        let polypoints = t1.imageproc_points();

        let color = ColorPicker::sample(&reference, center_x, center_y);

        let candidate = SImage {
            img: imageproc::drawing::draw_polygon(&current.img, &polypoints, color).into(),
        };

        // check the deltas from that region
        let reference_crop = reference.crop(&region);
        let current_crop = current.crop(&region);
        let candidate_crop = candidate.crop(&region);

        let current_delta = reference_crop.delta(&current_crop.img);
        let candidate_delta = reference_crop.delta(&candidate_crop.img);

        if candidate_delta < current_delta {
            current = candidate;
            successes_at_radius += 1.0;
        }
    }

    // UI run loop; doesn't exit.
    // main_window::run();
}

pub struct PointSelector {
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
