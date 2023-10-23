use serde::{Deserialize, Serialize};

use crate::Region;
use image::Rgba;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
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

    fn center_to_center_distance(&self, other: &Circle) -> f32 {
        let x_diff = self.x as f32 - other.x as f32;
        let y_diff = self.y as f32 - other.y as f32;

        (x_diff.powi(2) + y_diff.powi(2)).sqrt()
    }

    pub fn overlaps_circle(&self, other: &Circle) -> bool {
        self.center_to_center_distance(other) < (self.radius + other.radius) as f32
    }

    pub fn overlaps_region(&self, region: &Region) -> bool {
        let cx = self.x as f32;
        let cy = self.y as f32;

        let rx = region.min_x as f32;
        let ry = region.min_y as f32;
        let rw = region.max_x as f32 - region.min_x as f32;
        let rh = region.max_y as f32 - region.min_y as f32;

        let test_x = if cx < rx {
            rx
        } else if cx > rx + rw {
            rx + rw
        } else {
            cx
        };
        let test_y = if cy < ry {
            ry
        } else if cy > ry + rh {
            ry + rh
        } else {
            cy
        };
        let dist_x = cx - test_x;
        let dist_y = cy - test_y;
        let distance = (dist_x.powi(2) + dist_y.powi(2)).sqrt();

        distance <= self.radius as f32
    }
}
