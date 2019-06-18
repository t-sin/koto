use std::sync::{Arc, Mutex};

use super::super::time::Time;

pub type Signal = (f64, f64);
pub type Amut<T> = Arc<Mutex<T>>;

pub trait Unit {
    fn proc(&mut self, time: &Time) -> Signal;
}

pub trait Osc: Unit {
    fn set_freq(&mut self, freq: AUnit);
}

pub enum ADSR {
    Attack,
    Decay,
    Sustin,
    Release,
    None,
}

pub trait Eg: Unit {
    fn set_state(&mut self, state: ADSR, eplaced: u64);
}

pub enum Node {
    Val(f64),
    Sig(Amut<Unit + Send>),
    Osc(Amut<Osc + Send>),
    Eg(Amut<Eg + Send>),
}

pub struct UnitGraph {
    pub last_tick: u64,
    pub last_sig: Signal,
    pub node: Node,
}

impl UnitGraph {
    pub fn new(node: Node) -> UnitGraph {
        UnitGraph {
            last_tick: 0,
            last_sig: (0.0, 0.0),
            node: node,
        }
    }
}

pub type AUnit = Amut<UnitGraph>;

impl Unit for Node {
    fn proc(&mut self, time: &Time) -> Signal {
        match self {
            Node::Val(v) => (*v, *v),
            Node::Sig(u) => u.lock().unwrap().proc(time),
            Node::Osc(u) => u.lock().unwrap().proc(time),
            Node::Eg(u) => u.lock().unwrap().proc(time),
        }
    }
}

impl Osc for Node {
    fn set_freq(&mut self, freq: Amut<UnitGraph>) {
        match self {
            Node::Osc(u) => u.lock().unwrap().set_freq(freq),
            _ => (),
        }
    }
}

impl Eg for Node {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        match self {
            Node::Eg(u) => u.lock().unwrap().set_state(state, eplaced),
            _ => (),
        }
    }
}

impl Unit for UnitGraph {
    fn proc(&mut self, time: &Time) -> Signal {
        if self.last_tick < time.tick {
            self.last_sig = self.node.proc(&time);
            self.last_tick = time.tick;
            self.last_sig
        } else {
            self.last_sig
        }
    }
}
