use serde::{Deserialize, Serialize};

use image::Rgba;

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
