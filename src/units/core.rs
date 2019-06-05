use super::super::time::Time;
use super::unit::Stateful;
use super::unit::Signal1;
use super::unit::Unit1;

pub struct Offset {
    pub v: f64,
    pub src: Unit1,
}

impl Signal1 for Offset {
    fn calc1(&self, time: &Time) -> f64 {
        self.src.calc1(&time) + self.v
    }
}

impl Stateful for Offset {
    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}

pub struct Gain {
    pub v: f64,
    pub src: Unit1,
}

impl Signal1 for Gain {
    fn calc1(&self, time: &Time) -> f64 {
        self.src.calc1(&time) * self.v
    }
}

impl Stateful for Gain {
    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}
