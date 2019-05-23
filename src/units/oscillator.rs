use super::super::time::Time;
use super::unit::Unit;

pub struct Sine {
    pub init_ph: f64,
    pub ph: f64,
    pub freq: f64,
}

impl Unit for Sine {
    fn calc(&self, _time: &Time) -> f64 {
        self.ph.sin()
    }

    fn update(&mut self, time: &Time) {
        self.ph += self.freq / time.sample_rate * std::f64::consts::PI;
    }
}
