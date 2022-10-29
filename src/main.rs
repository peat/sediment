mod main_window;

// use image::{DynamicImage, Pixel, Rgba, RgbaImage};

fn main() {
    // UI run loop; doesn't exit.
    main_window::run();
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
struct Region {
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
    x: usize,
    y: usize,
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

        // adjust the indexes ...

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
