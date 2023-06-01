use core::cell::RefCell;

pub struct FrameStats(RefCell<RawFrameStats>);
impl FrameStats {
    pub fn new() -> Self {
        Self(RefCell::new(RawFrameStats {
            last_frame_time: 0.,
            frame_time_delta: 0.,
            frame_count: 0,
            frames_per_second: 0.,
        }))
    }

    pub fn frame_time_delta(&self) -> f64 {
        let raw = self.0.borrow();
        raw.frame_time_delta
    }

    pub fn frames_per_second(&self) -> f64 {
        let raw = self.0.borrow();
        raw.frames_per_second
    }

    pub fn update(&mut self, time: f64) {
        let mut raw = self.0.borrow_mut();
        raw.update(time);
    }
}

#[derive(Clone, Copy)]
struct RawFrameStats {
    last_frame_time: f64,
    frame_time_delta: f64,
    frames_per_second: f64,
    frame_count: usize,
}
impl RawFrameStats {
    fn update(&mut self, time: f64) {
        self.frame_time_delta = time - self.last_frame_time;
        self.last_frame_time = time;
        self.frame_count += 1;

        if self.frame_time_delta == 0.0 || self.frame_count < 5 {
            return;
        }

        let frames_per_second = 1.0 / self.frame_time_delta;
        let count = self.frame_count.min(50);

        // Knuth algorithm for updating an average.
        self.frames_per_second += (frames_per_second - self.frames_per_second) / count as f64;
    }
}
