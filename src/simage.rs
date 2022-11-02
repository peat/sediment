use image::{DynamicImage, GenericImageView, Rgba};

use crate::Region;

pub struct SImage {
    pub img: DynamicImage,
}

impl SImage {
    pub fn new(width: u32, height: u32) -> Self {
        let mut img = DynamicImage::new_rgba8(width, height);

        for pixel in img.as_mut_rgba8().unwrap().pixels_mut() {
            *pixel = Rgba([0, 0, 0, 255]);
        }

        Self { img }
    }

    pub fn open(path: &str) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("{}", e))?;

        Ok(Self { img })
    }

    pub fn width(&self) -> u32 {
        self.img.width()
    }

    pub fn height(&self) -> u32 {
        self.img.height()
    }

    pub fn crop(&self, region: &Region) -> SImage {
        let x = region.abs_x();
        let y = region.abs_y();
        let width = region.abs_width();
        let height = region.abs_height();

        let img = self.img.crop_imm(x, y, width, height);

        Self { img }
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
