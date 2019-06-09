use super::super::time::Time;

pub type Value = (f64, f64);

pub trait Stateful {
    fn update(&mut self, time: &Time);
}

pub trait Signal: Stateful {
    fn calc(&self, time: &Time) -> Value;
}


pub enum Unit {
    Value(f64),
    Unit(Box<Signal + Send>),
}

impl Signal for Unit {
    fn calc(&self, time: &Time) -> Value {
        match self {
            Unit::Value(v) => (*v, *v),
            Unit::Unit(u) => u.calc(&time),
        }
    }
}

impl Stateful for Unit {
    fn update(&mut self, time: &Time) {
        match self {
            Unit::Value(_v) => (),
            Unit::Unit(u) => u.update(&time),
        }
    }
}
