use crate::Canvas;
use image::GenericImageView;
use rand::Rng;

/// Picks a random point on the Canvas
pub struct RandomPointSelector {
    width: u32,
    height: u32,
}

impl RandomPointSelector {
    pub fn new(canvas: &Canvas) -> Self {
        Self {
            width: canvas.width(),
            height: canvas.height(),
        }
    }
}

impl Iterator for RandomPointSelector {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::thread_rng();

        let x = rng.gen_range(0..self.width);
        let y = rng.gen_range(0..self.height);

        Some((x, y))
    }
}

/// Picks the point with the greatest difference from the reference image
pub struct DistancePointSelector {
    points: Vec<(usize, u32, u32)>,
}

impl DistancePointSelector {
    #[allow(dead_code)]
    pub fn new(reference: &Canvas, current: &Canvas, limit: usize) -> Self {
        let mut all_points: Vec<(usize, u32, u32)> =
            std::iter::zip(reference.img.pixels(), current.img.pixels())
                .map(|(a, b)| (Canvas::pixel_delta(a.2, b.2), a.0, a.1)) // 0 and 1 are coordinates; 2 is the pixel
                .collect();

        all_points.sort_by(|a, b| a.0.cmp(&b.0));

        let points = all_points[(all_points.len() - limit)..all_points.len()].to_vec();

        Self { points }
    }
}

impl Iterator for DistancePointSelector {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        let output = self.points.pop()?;
        Some((output.1, output.2))
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;

    use super::DistancePointSelector;

    #[test]
    fn distance_point_selector() {
        let reference = Canvas::open("./teapot.jpg").unwrap();
        let current = Canvas::new(reference.width(), reference.height());
        let limit = 1000;

        let mut dps = DistancePointSelector::new(&reference, &current, limit);

        let first = dps.points.first().unwrap().clone();
        let last = dps.points.last().unwrap().clone();

        // should be reverse sorted; last() gets popped off the stack first with next()
        assert!(first.0 < last.0);
        assert_eq!(dps.points.len(), limit);
        assert_eq!(dps.next(), Some((last.1, last.2)));
        assert_eq!(dps.points.len(), limit - 1);
    }
}
