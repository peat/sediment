use image::{DynamicImage, GenericImageView, Rgba};

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
    pub source: DynamicImage,
    pub candidate: DynamicImage,
    pub current: DynamicImage,
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
