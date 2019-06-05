use super::super::time::Time;
use super::unit::Stateful;
use super::unit::Signal1;
use super::unit::Unit1;

pub struct Sine {
    pub init_ph: Unit1,
    pub ph: f64,
    pub freq: Unit1,
}

impl Signal1 for Sine {
    fn calc1(&self, time: &Time) -> f64 {
        (self.init_ph.calc1(&time) + self.ph).sin()
    }
}

impl Stateful for Sine {
    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc1(&time) / time.sample_rate as f64 * std::f64::consts::PI;
    }
}
