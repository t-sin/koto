use super::super::time::Time;

pub trait Calc {
    fn calc(&self, time: &Time) -> f64;
    fn update(&mut self, time: &Time);
}

pub enum Unit {
    Value(f64),
    Unit(Box<Calc + Send>),
    Units(Vec<Box<Calc + Send>>),
}

impl Calc for Unit {
    fn calc(&self, time: &Time) -> f64 {
        match self {
            Unit::Value(v) => *v,
            Unit::Unit(u) => u.calc(&time),
            Unit::Units(us) =>  us.iter().fold(0.0, |acc, s| acc + s.calc(&time)),
        }
    }

    fn update(&mut self, time: &Time) {
        match self {
            Unit::Value(_v) => (),
            Unit::Unit(u) => u.update(&time),
            Unit::Units(us) => us.iter_mut().for_each(move |s| s.update(&time)),
        }
    }
}
