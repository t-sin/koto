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

pub struct AMix {
    pub sources: Vec<Box<Signal>>,
}

impl Signal for AMix {
    fn calc(&self, time: &Time) -> Value {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter() {
            let (l2, r2) = u.calc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }
}

impl Stateful for AMix {
    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.update(&time);
        }
    }
}

pub struct MMix {
    pub sources: Vec<Box<Signal>>,
}

impl Signal for MMix {
    fn calc(&self, time: &Time) -> Value {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter() {
            let (l2, r2) = u.calc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }
}

impl Stateful for MMix {
    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.update(&time);
        }
    }
}
