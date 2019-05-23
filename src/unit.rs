use super::time;

pub trait Unit {
    fn calc(&self, time: &time::Time) -> f64;
    fn update(&mut self, time: &time::Time);
}

pub struct Osc {
    pub init_ph: f64,
    pub ph: f64,
    pub freq: f64,
}

impl Unit for Osc {
    fn calc(&self, _time: &time::Time) -> f64 {
        self.ph.sin()
    }

    fn update(&mut self, time: &time::Time) {
        self.ph += self.freq / time.sample_rate * std::f64::consts::PI;
    }
}
