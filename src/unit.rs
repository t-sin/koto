use super::clock;

pub trait Unit {
    fn calc(self, clock: &clock::Clock) -> f64;
    fn update(&mut self, clock: &clock::Clock);
}

pub struct Osc {
    init_ph: f64,
    ph: f64,
    freq: f64,
}

impl Unit for Osc {
    fn calc(self, _clock: &clock::Clock) -> f64 {
        self.ph.sin()
    }

    fn update(&mut self, clock: &clock::Clock) {
        self.ph += self.freq / clock.sample_rate * std::f64::consts::PI;
    }
}
