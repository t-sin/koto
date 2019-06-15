use super::super::time::Time;
use super::unit::Signal;
use super::unit::{Unit, AUnit};

pub struct Pan {
    pub v: AUnit,
    pub src: AUnit,
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
    pub src: AUnit,
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
    pub src: AUnit,
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

pub struct Add {
    pub sources: Vec<AUnit>,
}

impl Unit for Add {
    fn calc(&self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter() {
            let (l2, r2) = u.lock().unwrap().calc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }

    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.lock().unwrap().update(&time);
        }
    }
}

pub struct Multiply {
    pub sources: Vec<AUnit>,
}

impl Unit for Multiply {
    fn calc(&self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter() {
            let (l2, r2) = u.lock().unwrap().calc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }

    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.lock().unwrap().update(&time);
        }
    }
}
