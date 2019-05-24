use super::super::time::Time;
use super::unit::Calc;
use super::unit::Unit;

pub struct Offset {
    pub v: f64,
    pub src: Unit,
}

impl Calc for Offset {
    fn calc(&self, time: &Time) -> f64 {
        self.src.calc(&time) + self.v
    }

    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}

pub struct Gain {
    pub v: f64,
    pub src: Unit,
}

impl Calc for Gain {
    fn calc(&self, time: &Time) -> f64 {
        self.src.calc(&time) * self.v
    }

    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}
