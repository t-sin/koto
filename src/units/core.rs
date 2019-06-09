use super::super::time::Time;
use super::unit::Value;
use super::unit::Signal;
use super::unit::Unit;

pub struct Pan<'a> {
    pub v: &'a Unit<'a>,
    pub src: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Pan<'a> {
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

    fn update(&mut self, time: &Time) {
        self.v.update(&time);
        self.src.update(&time);
    }
}

pub struct Offset<'a> {
    pub v: f64,
    pub src: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Offset<'a> {
    fn calc(&self, time: &Time) -> Value {
        let (l, r) = self.src.calc(&time);
        (l + self.v, r + self.v)
    }

    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}

pub struct Gain<'a> {
    pub v: f64,
    pub src: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Gain<'a> {
    fn calc(&self, time: &Time) -> Value {
        let (l, r) = self.src.calc(&time);
        (l * self.v, r * self.v)
    }

    fn update(&mut self, time: &Time) {
        self.src.update(&time);
    }
}

pub struct AMix<'a> {
    pub sources: Vec<Box<Signal<'a>>>,
}

impl<'a> Signal<'a> for AMix<'a> {
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

    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.update(&time);
        }
    }
}

pub struct MMix<'a> {
    pub sources: Vec<Box<Signal<'a>>>,
}

impl<'a> Signal<'a> for MMix<'a> {
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

    fn update(&mut self, time: &Time) {
        for u in self.sources.iter_mut() {
            u.update(&time);
        }
    }
}
