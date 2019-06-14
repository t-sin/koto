use std::sync::{Arc, Mutex};

use super::super::time::Time;

pub type Signal = (f64, f64);
pub type Amut<T> = Arc<Mutex<T>>;

pub trait Unit {
    fn calc(&self, time: &Time) -> Signal;
    fn update(&mut self, time: &Time);
}

pub enum UType {
    Sig(Amut<Unit + Send>),
    Osc(Amut<Osc + Send>),
}

pub enum UnitGraph {
    Value(f64),
    Unit(UType),
}

pub type AUnit = Amut<UnitGraph>;

pub trait Osc: Unit {
    fn set_freq(&mut self, freq: Arc<Mutex<UnitGraph>>);
}

impl Unit for UType {
    fn calc(&self, time: &Time) -> Signal {
        match self {
            UType::Sig(u) => u.lock().unwrap().calc(time),
            UType::Osc(u) => u.lock().unwrap().calc(time),
        }
    }

    fn update(&mut self, time: &Time) {
        match self {
            UType::Sig(u) => u.lock().unwrap().update(time),
            UType::Osc(u) => u.lock().unwrap().update(time),
        }
    }
}

impl Osc for UType {
    fn set_freq(&mut self, freq: Amut<UnitGraph>) {
        match self {
            UType::Sig(_u) => (),
            UType::Osc(u) => u.lock().unwrap().set_freq(freq),
        }
    }
}

impl Unit for UnitGraph {
    fn calc(&self, time: &Time) -> Signal {
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
