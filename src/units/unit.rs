use super::super::time::Time;

pub trait Stateful {
    fn update(&mut self, time: &Time);
}

pub trait Signal1: Stateful {
    fn calc1(&self, time: &Time) -> f64;
}

pub trait Signal2: Stateful {
    fn calc2(&self, time: &Time) -> (f64, f64);
}


pub enum Unit1 {
    Value(f64),
    Unit(Box<Signal1 + Send>),
    Units(Vec<Box<Signal1 + Send>>),
}

impl Signal1 for Unit1 {
    fn calc1(&self, time: &Time) -> f64 {
        match self {
            Unit1::Value(v) => *v,
            Unit1::Unit(u) => u.calc1(&time),
            Unit1::Units(us) =>  us.iter().fold(0.0, |acc, s| acc + s.calc1(&time)),
        }
    }
}

impl Stateful for Unit1{
    fn update(&mut self, time: &Time) {
        match self {
            Unit1::Value(_v) => (),
            Unit1::Unit(u) => u.update(&time),
            Unit1::Units(us) => us.iter_mut().for_each(move |s| s.update(&time)),
        }
    }
}
