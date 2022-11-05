pub struct RateMeter {
    limit: usize,
    samples: Vec<usize>,
}

impl RateMeter {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            samples: vec![],
        }
    }

    pub fn sample(&mut self, value: usize) {
        self.samples.push(value);

        if self.samples.len() > self.limit {
            self.samples.remove(0);
        }
    }

    pub fn rate(&self) -> Option<f32> {
        if self.samples.len() >= self.limit {
            let sum: usize = self.samples.iter().sum();
            let rate = (sum as f32) / (self.limit as f32);
            Some(rate)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.samples = vec![];
    }

    /// Defaults false if there aren't enough samples
    pub fn is_below(&self, rate: f32) -> bool {
        if let Some(current_rate) = self.rate() {
            current_rate < rate
        } else {
            false
        }
    }
}
