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
    Units(Vec<Box<Signal + Send>>),
}

impl Signal for Unit {
    fn calc(&self, time: &Time) -> Value {
        match self {
            Unit::Value(v) => (*v, *v),
            Unit::Unit(u) => u.calc(&time),
            Unit::Units(us) => us.iter().fold((0.0, 0.0), |acc, s| {
                let (l1, r1) = acc;
                let (l2, r2) = s.calc(&time);
                (l1 + l2, r1 + r2)
            }),
        }
    }
}

impl Stateful for Unit {
    fn update(&mut self, time: &Time) {
        match self {
            Unit::Value(_v) => (),
            Unit::Unit(u) => u.update(&time),
            Unit::Units(us) => us.iter_mut().for_each(move |s| s.update(&time)),
        }
    }
}
