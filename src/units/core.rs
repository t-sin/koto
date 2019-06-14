use std::sync::{Arc, Mutex};

use super::super::time::Time;
use super::unit::Signal;
use super::unit::Unit;
use super::unit::UType;
use super::unit::UnitGraph;

pub struct Pan {
    pub v: Arc<Mutex<UnitGraph>>,
    pub src: Arc<Mutex<UnitGraph>>,
}

impl Unit for Pan {
    fn calc(&self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().calc(&time);
        let v = self.v.lock().unwrap().calc(&time).0;
        if v > 0.0 {
            (l * (1.0 - v), r)
        } else if v < 0.0 {
            (l, r * (1.0 - v))
        } else {
            (l, r)
        }
    }

    fn update(&mut self, time: &Time) {
        self.v.lock().unwrap().update(&time);
        self.src.lock().unwrap().update(&time);
    }
}

pub struct Offset {
    pub v: f64,
    pub src: Arc<Mutex<UnitGraph>>,
}

impl Unit for Offset {
    fn calc(&self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().calc(&time);
        (l + self.v, r + self.v)
    }

    fn update(&mut self, time: &Time) {
        self.src.lock().unwrap().update(&time);
    }
}

pub struct Gain {
    pub v: f64,
    pub src: Arc<Mutex<UnitGraph>>,
}

impl Unit for Gain {
    fn calc(&self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().calc(&time);
        (l * self.v, r * self.v)
    }

    fn update(&mut self, time: &Time) {
        self.src.lock().unwrap().update(&time);
    }
}

pub struct AMix {
    pub sources: Vec<Box<Unit>>,
}

impl Unit for AMix {
    fn calc(&self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter() {
            let (l2, r2) = u.calc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }

    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.update(&time);
        }
    }
}

pub struct MMix {
    pub sources: Vec<Box<Unit>>,
}

impl Unit for MMix {
    fn calc(&self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter() {
            let (l2, r2) = u.calc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }

    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.update(&time);
        }
    }
}
