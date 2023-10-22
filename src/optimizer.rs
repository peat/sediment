use crate::circle::Circle;
use crate::render::Render;
use crate::Canvas;
use std::collections::VecDeque;
use std::time::Instant;

pub struct Optimizer {
    circles: Vec<Circle>,
    reference: Canvas,
}

impl Optimizer {
    pub fn new(circles: Vec<Circle>) -> Self {
        let reference = Render::render_raster(&circles);
        Self { circles, reference }
    }

    /// Removes all circles that have no impact on the final image
    pub fn prune(&self) -> Vec<Circle> {
        eprintln!("Pruning {} circles ...", self.circles.len());

        let mut kept_circles = vec![];
        let mut discarded_circles = vec![];
        let mut untested_circles = VecDeque::from(self.circles.clone());
        let mut progressive_canvas = Render::create_empty_canvas(&self.circles);

        while let Some(next_circle) = untested_circles.pop_front() {
            let iteration_time = Instant::now();
            let mut new_canvas = progressive_canvas.clone();

            // add the remainder of the circles to the new_canvas
            let render_timer = Instant::now();
            for c in untested_circles.iter() {
                Render::add_raster_circle(&mut new_canvas, c);
            }
            let render_time = render_timer.elapsed();

            // if the new canvas is the same as the reference, then this circle is redundant
            let compare_timer = Instant::now();
            if new_canvas.is_equal(&self.reference) {
                discarded_circles.push(next_circle);
            } else {
                Render::add_raster_circle(&mut progressive_canvas, &next_circle);
                kept_circles.push(next_circle);
            }
            let compare_time = compare_timer.elapsed();

            eprintln!(
                "  {} circles remaining, {} discarded (iteration: {:?}, render: {:?}, compare: {:?})",
                untested_circles.len(),
                discarded_circles.len(),
                iteration_time.elapsed(),
                render_time,
                compare_time,
            );
        }

        kept_circles
    }
}
