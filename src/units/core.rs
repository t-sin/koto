use super::super::time::Time;
use super::unit::Value;
use super::unit::Stateful;
use super::unit::Signal;
use super::unit::Unit;

pub struct Offset {
    pub v: f64,
    pub src: Unit,
}

impl Signal for Offset {
    fn calc(&self, time: &Time) -> Value {
        let (l, r) = self.src.calc(&time);
        (l + self.v, r + self.v)
    }
}

impl Stateful for Offset {
    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}

pub struct Gain {
    pub v: f64,
    pub src: Unit,
}

impl Signal for Gain {
    fn calc(&self, time: &Time) -> Value {
        let (l, r) = self.src.calc(&time);
        (l * self.v, r * self.v)
    }
}

impl Stateful for Gain {
    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}
