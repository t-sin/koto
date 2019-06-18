extern crate num;

use super::super::time::Time;

use super::unit::Signal;
use super::unit::{Dump, Unit, AUnit};

pub struct Pan {
    pub v: AUnit,
    pub src: AUnit,
}

impl Unit for Pan {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().proc(&time);
        let v = self.v.lock().unwrap().proc(&time).0;

        if v > 0.0 {
            (l * (1.0 - v), r)
        } else if v < 0.0 {
            (l, r * (1.0 - v))
        } else {
            (l, r)
        }
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        vec.push(Box::new(self.v.lock().unwrap().dump()));
        vec.push(Box::new(self.src.lock().unwrap().dump()));
        Dump::Params(vec)
    }
}

pub struct Clip {
    pub min: f64,
    pub max: f64,
    pub src: AUnit,
}

impl Unit for Clip {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().proc(&time);
        (num::clamp(l, self.min, self.max), num::clamp(r, self.min, self.max))
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        vec.push(Box::new(Dump::Str(self.min.to_string())));
        vec.push(Box::new(Dump::Str(self.max.to_string())));
        vec.push(Box::new(self.src.lock().unwrap().dump()));
        Dump::Params(vec)
    }
}

pub struct Offset {
    pub v: f64,
    pub src: AUnit,
}

impl Unit for Offset {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().proc(&time);
        (l + self.v, r + self.v)
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        vec.push(Box::new(Dump::Str(self.v.to_string())));
        vec.push(Box::new(self.src.lock().unwrap().dump()));
        Dump::Params(vec)
    }
}

pub struct Gain {
    pub v: f64,
    pub src: AUnit,
}

impl Unit for Gain {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.lock().unwrap().proc(&time);
        (l * self.v, r * self.v)
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        vec.push(Box::new(Dump::Str(self.v.to_string())));
        vec.push(Box::new(self.src.lock().unwrap().dump()));
        Dump::Params(vec)
    }
}

pub struct Add {
    pub sources: Vec<AUnit>,
}

impl Unit for Add {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.lock().unwrap().proc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        for u in self.sources.iter() {
            vec.push(Box::new(u.lock().unwrap().dump()));
        }
        Dump::Params(vec)
    }
}

pub struct Multiply {
    pub sources: Vec<AUnit>,
}

impl Unit for Multiply {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.lock().unwrap().proc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        for u in self.sources.iter() {
            vec.push(Box::new(u.lock().unwrap().dump()));
        }
        Dump::Params(vec)
    }
}
