use std::sync::{Arc, Mutex};

use super::super::time::Time;

pub type Value = (f64, f64);

pub trait Signal<'a> {
    fn calc(&self, time: &Time) -> Value;
    fn update(&mut self, time: &Time);
    fn set_freq(&mut self, _u: &'a Unit<'a>) {}
}

pub enum Unit<'a> {
    Value(f64),
    Unit(Arc<Mutex<Signal<'a> + Send>>),
}

impl<'a> Signal<'a> for Unit<'a> {
    fn calc(&self, time: &Time) -> Value {
        match self {
            Unit::Value(v) => (*v, *v),
            Unit::Unit(u) => {
                u.lock().unwrap().calc(&time)
            },
        }
    }

    fn update(&mut self, time: &Time) {
        match self {
            Unit::Value(_v) => (),
            Unit::Unit(u) => u.lock().unwrap().update(&time),
        }
    }
}
