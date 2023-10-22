use image::{DynamicImage, GenericImageView, Rgba};

use crate::{Circle, Region};

#[derive(Clone)]
pub struct Canvas {
    pub center_x: i32,
    pub center_y: i32,
    pub img: DynamicImage,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let mut img = DynamicImage::new_rgba8(width, height);

        // make our new image black
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

    pub fn section(&self, region: &Region) -> Canvas {
        let x = region.real_origin_x();
        let y = region.real_origin_y();
        let width = region.real_width();
        let height = region.real_height();

        // get the cropped image section
        let img = self.img.crop_imm(x, y, width, height);

        Self {
            center_x: region.real_center_x() as i32,
            center_y: region.real_center_y() as i32,
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

    pub fn pixel_delta(a: Rgba<u8>, b: Rgba<u8>) -> usize {
        let mut delta = 0;

        delta += Self::channel_delta(a[0], b[0]) as usize;
        delta += Self::channel_delta(a[1], b[1]) as usize;
        delta += Self::channel_delta(a[2], b[2]) as usize;
        delta
    }

    pub fn byte_delta(a: u8, b: u8) -> usize {
        if a >= b {
            (a - b) as usize
        } else {
            (b - a) as usize
        }
    }

    pub fn delta(&self, img: &DynamicImage) -> usize {
        std::iter::zip(self.img.as_bytes(), img.as_bytes())
            .map(|(a, b)| Self::byte_delta(*a, *b)) // 0 and 1 are coordinates; 2 is the pixel
            .sum()
    }

    pub fn is_equal(&self, other: &Canvas) -> bool {
        for (a, b) in std::iter::zip(self.img.as_bytes(), other.img.as_bytes()) {
            if a != b {
                return false;
            }
        }
        true
    }

    pub fn draw_circle(&mut self, circle: &Circle) {
        let color = Rgba::from([circle.r, circle.g, circle.b, 255]);

        imageproc::drawing::draw_filled_circle_mut(
            &mut self.img,
            (circle.x as i32, circle.y as i32),
            circle.radius as i32,
            color,
        );
    }

    pub fn save(&self, path: &str) {
        self.img.save(path).unwrap();
    }
}
