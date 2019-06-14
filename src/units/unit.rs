use std::sync::{Arc, Mutex};

use super::super::time::Time;

pub type Value = (f64, f64);

pub trait Signal {
    fn calc(&self, time: &Time) -> Value;
    fn update(&mut self, time: &Time);
}

pub enum Unit {
    Sig(Arc<Mutex<Signal + Send>>),
    Osc(Arc<Mutex<Osc + Send>>),
}

pub enum UnitGraph {
    Value(f64),
    Unit(Unit),
}

pub trait Osc: Signal {
    fn set_freq(&mut self, freq: Arc<Mutex<UnitGraph>>);
}

impl Signal for Unit {
    fn calc(&self, time: &Time) -> Value {
        match self {
            Unit::Sig(u) => u.lock().unwrap().calc(time),
            Unit::Osc(u) => u.lock().unwrap().calc(time),
        }
    }

    fn update(&mut self, time: &Time) {
        match self {
            Unit::Sig(u) => u.lock().unwrap().update(time),
            Unit::Osc(u) => u.lock().unwrap().update(time),
        }
    }
}

impl Osc for Unit {
    fn set_freq(&mut self, freq: Arc<Mutex<UnitGraph>>) {
        match self {
            Unit::Sig(_u) => (),
            Unit::Osc(u) => u.lock().unwrap().set_freq(freq),
        }
    }
}

impl Signal for UnitGraph {
    fn calc(&self, time: &Time) -> Value {
        match self {
            UnitGraph::Value(v) => (*v, *v),
            UnitGraph::Unit(u) => u.calc(&time),
        }
    }

    fn update(&mut self, time: &Time) {
        match self {
            UnitGraph::Value(_v) => (),
            UnitGraph::Unit(u) => u.update(&time),
        }
    }
}
