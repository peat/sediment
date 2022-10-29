use image::{DynamicImage, GenericImageView, Rgba};
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
    println!("Rate: {}/ms", (comparisons as u128) / elapsed.as_millis());

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

#[derive(Debug)]
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

    fn delta(a: Rgba<u8>, b: Rgba<u8>) -> usize {
        // convert to RGB without the alpha channel so that we can compare
        // the visible values

        let mut delta = 0;

        delta += Self::channel_delta(a[0], b[0]) as usize;
        delta += Self::channel_delta(a[1], b[1]) as usize;
        delta += Self::channel_delta(a[2], b[2]) as usize;
        delta
    }

    pub fn deltas(&self) -> ImageDeltas {
        let mut deltas = ImageDeltas::new();

        deltas.candidate = std::iter::zip(self.source.pixels(), self.candidate.pixels())
            .map(|(a, b)| Self::delta(a.2, b.2))
            .sum();
        deltas.current = std::iter::zip(self.source.pixels(), self.current.pixels())
            .map(|(a, b)| Self::delta(a.2, b.2))
            .sum();

        deltas
    }
}

#[derive(Debug)]
pub struct Region {
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    index_x: usize,
    index_y: usize,
}

impl Region {
    pub fn new(start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Self {
        let index_x = start_x;
        let index_y = start_y;

        Self {
            start_x,
            start_y,
            end_x,
            end_y,
            index_x,
            index_y,
        }
    }

    pub fn width(&self) -> usize {
        (self.end_x - self.start_x) + 1
    }

    pub fn height(&self) -> usize {
        (self.end_y - self.start_y) + 1
    }

    pub fn len(&self) -> usize {
        self.width() * self.height()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    // translate an X,Y coordinate into a position index for an image with a given width
    pub fn index(&self, width: usize) -> usize {
        (self.y * width) + self.x
    }
}

impl Iterator for Region {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        // bounds checking
        if self.index_y > self.end_y {
            return None;
        }

        // already computed in the previous pass
        let output = Point {
            x: self.index_x,
            y: self.index_y,
        };

        // check to see if we head to the next row
        if self.index_x == self.end_x {
            self.index_x = self.start_x;
            self.index_y += 1;
        } else {
            self.index_x += 1;
        }

        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_index() {
        let expected = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24,
        ];

        let mut collected = vec![];

        let dimension = 5;

        for y in 0..dimension {
            for x in 0..dimension {
                let p = Point { x, y };
                collected.push(p.index(dimension));
            }
        }

        assert_eq!(collected, expected);
    }

    #[test]
    fn test_region_iterator() {
        // full region
        let mut r1 = Region::new(0, 0, 4, 4);
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(r1.next(), Some(Point { x, y }));
            }
        }

        assert_eq!(r1.next(), None);
    }

    #[test]
    fn test_region_dimensions() {
        let r1 = Region::new(0, 0, 4, 4);
        assert_eq!(r1.len(), 25);
        assert_eq!(r1.width(), 5);
        assert_eq!(r1.height(), 5);

        // this is a one pixel region at 5,5
        let r2 = Region::new(5, 5, 5, 5);
        assert_eq!(r2.len(), 1);
        assert_eq!(r2.width(), 1);
        assert_eq!(r2.height(), 1);
    }
}
