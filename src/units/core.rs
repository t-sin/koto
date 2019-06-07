use super::super::time::Time;
use super::unit::Value;
use super::unit::Stateful;
use super::unit::Signal;
use super::unit::Unit;

pub struct Pan {
    pub v: Unit,
    pub src: Unit,
}

impl Signal for Pan {
    fn calc(&self, time: &Time) -> Value {
        let (l, r) = self.src.calc(&time);
        let v = self.v.calc(&time).0;
        if v > 0.0 {
            (l * (1.0 - v), r)
        } else if v < 0.0 {
            (l, r * (1.0 - v))
        } else {
            (l, r)
        }
    }
}

impl Stateful for Pan {
    fn update(&mut self, time: &Time) {
        self.v.update(&time);
        self.src.update(&time);
    }
}

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
