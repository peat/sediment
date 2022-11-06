use image::{DynamicImage, GenericImageView, Rgba};

use crate::builder::Region;

#[derive(Clone)]
pub struct SImage {
    pub center_x: i32,
    pub center_y: i32,
    pub img: DynamicImage,
}

impl SImage {
    pub fn new(width: u32, height: u32) -> Self {
        let mut img = DynamicImage::new_rgba8(width, height);

        for pixel in img.as_mut_rgba8().unwrap().pixels_mut() {
            *pixel = Rgba([0, 0, 0, 255]);
        }

        Self {
            center_x: 0,
            center_y: 0,
            img,
        }
    }

    pub fn open(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("{}", e))?;

        Ok(Self {
            center_x: (img.width() as i32) / 2,
            center_y: (img.height() as i32) / 2,
            img,
        })
    }

    pub fn width(&self) -> u32 {
        self.img.width()
    }

    pub fn height(&self) -> u32 {
        self.img.height()
    }

    pub fn section(&self, region: &Region) -> SImage {
        let x = region.abs_x();
        let y = region.abs_y();
        let width = region.abs_width();
        let height = region.abs_height();

        let img = self.img.crop_imm(x, y, width, height);
        let mut center_x = region.radius as i32;
        let mut center_y = region.radius as i32;

        // re-determine the center point; adjust center_x and center_y for overhang
        if region.max_x > self.img.width() as i32 {
            center_x += region.max_x - self.img.width() as i32;
        }

        if region.min_x < 0 {
            center_x += region.min_x;
        }

        if region.max_y > self.img.height() as i32 {
            center_y += region.max_y - self.img.height() as i32;
        }

        if region.min_y < 0 {
            center_y += region.min_y;
        }

        Self {
            center_x,
            center_y,
            img,
        }
    }

    pub fn value(&self) -> usize {
        let mut value: usize = 0;
        for p in self.img.pixels() {
            value += Self::pixel_value(p.2); // 0 and 1 are coordinates, 2 is the pixel itself
        }
        value
    }

    fn pixel_value(a: Rgba<u8>) -> usize {
        let mut value: usize = 0;

        value += a[0] as usize;
        value += a[1] as usize;
        value += a[2] as usize;

        value
    }

    fn channel_delta(a: u8, b: u8) -> u8 {
        if a >= b {
            a - b
        } else {
            b - a
        }
    }

    fn pixel_delta(a: Rgba<u8>, b: Rgba<u8>) -> usize {
        let mut delta = 0;

        delta += Self::channel_delta(a[0], b[0]) as usize;
        delta += Self::channel_delta(a[1], b[1]) as usize;
        delta += Self::channel_delta(a[2], b[2]) as usize;
        delta
    }

    pub fn delta(&self, img: &DynamicImage) -> usize {
        std::iter::zip(self.img.pixels(), img.pixels())
            .map(|(a, b)| Self::pixel_delta(a.2, b.2)) // 0 and 1 are coordinates; 2 is the pixel
            .sum()
    }

    pub fn save(&self, path: &str) {
        self.img.save(path).unwrap();
    }
}
