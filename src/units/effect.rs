use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use super::super::time::Time;
use super::unit::{Mut, Signal, Walk, UDump, Dump, Unit, Node, UnitGraph, AUnit};

use super::super::tapirlisp::types::Env;

// filters

// delays

pub struct Delay {
    buffer: VecDeque<Box<Signal>>,
    time: AUnit,
    feedback: AUnit,
    mix: AUnit,
    src: AUnit,
}

impl Delay {
    pub fn new(time: AUnit, feedback: AUnit, mix: AUnit, src: AUnit, env: &Env) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Delay {
                buffer: VecDeque::with_capacity((env.time.sample_rate * 6) as usize),
                time: time,
                feedback: feedback,
                mix: mix,
                src: src
            })
        )))
    }
}

impl Walk for Delay {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.time) { self.time.0.lock().unwrap().walk(f); }
        if f(&self.feedback) { self.feedback.0.lock().unwrap().walk(f); }
        if f(&self.mix) { self.mix.0.lock().unwrap().walk(f); }
        if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
    }
}

impl Dump for Delay {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.time)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.time.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.feedback)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.feedback.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.mix)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.mix.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("delay".to_string(), vec)
    }
}

impl Unit for Delay {
    fn proc(&mut self, time: &Time) -> Signal {
        let (sl, sr) = self.src.0.lock().unwrap().proc(time);
        let mix = self.src.0.lock().unwrap().proc(time).0;
        let (l, r) = (sl * mix, sr * mix);
        (l, r)
    }
}
