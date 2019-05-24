use super::super::time::Time;
use super::unit::Calc;
use super::unit::Unit;

pub struct Sine {
    pub init_ph: Unit,
    pub ph: f64,
    pub freq: Unit,
}

impl Calc for Sine {
    fn calc(&self, time: &Time) -> f64 {
        (self.init_ph.calc(&time) + self.ph).sin()
    }

    fn update(&mut self, time: &Time) {
        self.ph += self.freq.calc(&time) / time.sample_rate as f64 * std::f64::consts::PI;
    }
}
