pub struct Time {
    pub channels: u32,
    pub sample_rate: u32,
    pub tick: u64,
}

pub trait Clock {
    fn update(&mut self);
}

impl Clock for Time {
    fn update(&mut self) {
        self.tick += 1;
    }
}
