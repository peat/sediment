use image::{DynamicImage, GenericImageView, Rgba};
use rand::Rng;
use std::time::Instant;

mod main_window;

// use image::{DynamicImage, Pixel, Rgba, RgbaImage};

fn main() {
    let source = ImageSet::open("teapot.jpg").unwrap();

    let start_time = Instant::now();
    let deltas = source.deltas();
    let stop_time = Instant::now();
    let elapsed = stop_time - start_time;

    let comparisons = source.source.pixels().count() * 2;

    println!("{:?}", deltas);
    println!("Comparisons: {}", comparisons);
    println!("Elapsed: {:?}", elapsed);
    println!(
        "Rate: {} pixels/ms",
        (comparisons as u128) / elapsed.as_millis()
    );

    // UI run loop; doesn't exit.
    // main_window::run();
}

/*

1. Open the source image
2. Create a blank target image
3. Analyze source image to find a region to work in
4. Create candidate region from the target region
5. Apply a change to the candidate region
6. Validate the change improves the candidate region vs target image
    - if it did not, go to 4
7. Apply candidate region to target image
8. Go to 3

*/

#[derive(Default, Debug)]
pub struct ImageDeltas {
    candidate: usize,
    current: usize,
}

impl ImageDeltas {
    pub fn new() -> Self {
        Self {
            candidate: 0,
            current: 0,
        }
    }
}

pub struct ImageSet {
    source: DynamicImage,
    candidate: DynamicImage,
    current: DynamicImage,
}

impl ImageSet {
    pub fn open(path: &str) -> Result<Self, String> {
        let source = image::open(path).map_err(|e| format!("{}", e))?;
        let candidate = DynamicImage::new_rgba8(source.width(), source.height());
        let current = candidate.clone();

        Ok(Self {
            source,
            candidate,
            current,
        })
    }

    pub fn width(&self) -> u32 {
        self.source.width()
    }

    pub fn height(&self) -> u32 {
        self.source.height()
    }

    pub fn region(&self, x: u32, y: u32, width: u32, height: u32) -> ImageSet {
        let source = self.source.crop_imm(x, y, width, height);
        let candidate = self.candidate.crop_imm(x, y, width, height);
        let current = self.current.crop_imm(x, y, width, height);

        Self {
            source,
            candidate,
            current,
        }
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

    pub fn deltas(&self) -> ImageDeltas {
        let mut deltas = ImageDeltas::new();

        deltas.candidate = std::iter::zip(self.source.pixels(), self.candidate.pixels())
            .map(|(a, b)| Self::pixel_delta(a.2, b.2)) // 0 and 1 are coordinates; 2 is the pixel
            .sum();
        deltas.current = std::iter::zip(self.source.pixels(), self.current.pixels())
            .map(|(a, b)| Self::pixel_delta(a.2, b.2))
            .sum();

        deltas
    }
}

pub struct PointSelector {
    width: u32,
    height: u32,
}

impl PointSelector {
    pub fn new(image_set: &ImageSet) -> Self {
        Self {
            width: image_set.width(),
            height: image_set.height(),
        }
    }

    pub fn point(&self) -> (u32, u32) {
        let mut rng = rand::thread_rng();

        let x = rng.gen_range(0, self.width);
        let y = rng.gen_range(0, self.height);

        (x, y)
    }
}
