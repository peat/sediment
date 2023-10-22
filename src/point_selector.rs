use crate::Canvas;
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
