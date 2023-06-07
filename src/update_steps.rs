#[derive(Clone, Copy)]
pub enum UpdateSteps {
    Initial,
    Update(usize),
    Clamped(usize),
}
impl UpdateSteps {
    pub fn count(&self) -> usize {
        match *self {
            UpdateSteps::Initial => 0,
            UpdateSteps::Update(count) => count,
            UpdateSteps::Clamped(count) => count,
        }
    }
}

pub struct UpdateStepTracker {
    last_step_time: Option<f64>,
    steps_per_second: f64,
    max_steps_per_update: usize,
}
impl UpdateStepTracker {
    pub fn new(steps_per_second: f64, max_steps_per_update: usize) -> Self {
        Self {
            last_step_time: None,
            steps_per_second,
            max_steps_per_update,
        }
    }
    pub fn steps_per_second(&self) -> f64 {
        self.steps_per_second
    }
    pub fn step_delta(&self) -> f64 {
        1.0 / self.steps_per_second
    }
    pub fn is_started(&self) -> bool {
        self.last_step_time.is_some()
    }
    pub fn update(&mut self, time: f64) -> UpdateSteps {
        let Some(last_update) = self.last_step_time else {
            self.last_step_time = Some(time);
            return UpdateSteps::Initial;
        };

        let update_count = ((time - last_update) * self.steps_per_second).floor() as usize;
        if update_count > self.max_steps_per_update {
            self.last_step_time = Some(time);
            UpdateSteps::Clamped(self.max_steps_per_update)
        } else {
            let count = update_count as f64;
            self.last_step_time = Some(last_update + count / self.steps_per_second);
            UpdateSteps::Update(update_count)
        }
    }
}
