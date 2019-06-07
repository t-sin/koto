use super::super::time::Time;
use super::unit::Value;
use super::unit::Stateful;
use super::unit::Signal;
use super::unit::Unit;

pub struct Sine {
    pub init_ph: Unit,
    pub ph: f64,
    pub freq: Unit,
}

impl Signal for Sine {
    fn calc(&self, time: &Time) -> Value {
        let init_ph = self.init_ph.calc(&time).0;
        let v = (init_ph + self.ph).sin();
        (v, v)
    }
}

impl Stateful for Sine {
    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64 * std::f64::consts::PI;
    }
}
