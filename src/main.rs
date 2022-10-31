use rand::Rng;
use std::time::Instant;

mod image_set;
mod main_window;

use image_set::ImageSet;

use image::GenericImageView;

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

        let x = rng.gen_range(0..self.width);
        let y = rng.gen_range(0..self.height);

        (x, y)
    }
}
