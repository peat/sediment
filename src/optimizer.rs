use crate::{Canvas, Circle, Region, Render};
use rayon::prelude::*;
use std::{
    fmt::Write,
    sync::mpsc::{channel, Sender},
    thread,
    time::Instant,
};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub struct Optimizer {
    circles: Vec<Circle>,
    reference: Canvas,
}

impl Optimizer {
    pub fn new(circles: Vec<Circle>) -> Self {
        let reference = Render::render_raster(&circles);
        Self { circles, reference }
    }

    pub fn parallel_prune(&self) -> Vec<Circle> {
        eprintln!("Pruning {} circles ...", self.circles.len());

        // start progress bar in it's own thread
        let (progress_tx, progress_rx) = channel();
        let target_count = self.circles.len();
        thread::spawn(move || {
            let mut count = 0;
            let pb = ProgressBar::new(target_count as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} (eta: {eta})",
                )
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                })
            );
            for _ in progress_rx.iter() {
                count += 1;
                pb.set_position(count as u64);
                if count == target_count {
                    pb.finish_with_message("Done");
                    return;
                }
            }
        });

        let timer = Instant::now();
        let pruned_circles: Vec<Circle> = self
            .circles
            .par_iter()
            .filter(|c| Self::test_circle(&self.reference, &self.circles, **c, progress_tx.clone()))
            .cloned()
            .collect();

        eprintln!(
            "Pruned to {} circles in {:?}",
            pruned_circles.len(),
            timer.elapsed()
        );

        pruned_circles
    }

    pub fn test_circle(
        reference: &Canvas,
        circles: &[Circle],
        candidate: Circle,
        progress: Sender<usize>,
    ) -> bool {
        // get the reference region that contains the candidate circle
        let candidate_region = Region::new(candidate.x, candidate.y, candidate.radius);

        // find all of the circles that overlap our candidate region
        let overlapping_circles: Vec<Circle> = circles
            .iter()
            .filter(|c| c.overlaps_region(&candidate_region))
            .cloned()
            .collect();

        let mut local_canvas = Render::create_empty_canvas(&overlapping_circles);
        for c in overlapping_circles.iter() {
            if c != &candidate {
                local_canvas.draw_circle(c);
            }
        }

        // get the regions that contains the candidate circle
        let reference_canvas = reference.section(&candidate_region);
        let test_canvas = local_canvas.section(&candidate_region);

        // if the canvases are equal, then the candidate circle is redundant
        let result = !test_canvas.is_equal(&reference_canvas);

        // update the counter
        progress.send(1).unwrap();

        result
    }
}
