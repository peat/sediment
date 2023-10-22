use std::collections::VecDeque;

pub struct RateMeter {
    limit: usize,
    samples: VecDeque<usize>,
}

impl RateMeter {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            samples: VecDeque::new(),
        }
    }

    pub fn sample(&mut self, value: usize) {
        self.samples.push_back(value);

        if self.samples.len() > self.limit {
            self.samples.pop_front();
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
        self.samples = VecDeque::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_samples() {
        let mut m = RateMeter::new(1);

        m.sample(1);
        assert_eq!(m.samples.len(), 1);

        m.sample(1);
        assert_eq!(m.samples.len(), 1);
    }

    #[test]
    fn averages_correctly() {
        let mut m = RateMeter::new(3);

        m.sample(1);
        m.sample(2);
        m.sample(3);
        assert_eq!(m.rate(), Some(2.0));

        m.sample(4);
        assert_eq!(m.rate(), Some(3.0));

        m.sample(5);
        assert_eq!(m.rate(), Some(4.0));
    }

    #[test]
    fn only_reports_if_full() {
        let mut m = RateMeter::new(3);
        m.sample(1);
        assert_eq!(m.rate(), None);

        m.sample(2);
        assert_eq!(m.rate(), None);

        m.sample(3);
        assert_eq!(m.rate(), Some(2.0));
    }
}
