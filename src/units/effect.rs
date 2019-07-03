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
        let len = (env.time.sample_rate * 6) as usize;
        let mut buffer = VecDeque::with_capacity(len);
        for _n in 0..len {
            buffer.push_back(Box::new((0.0, 0.0)));
        }
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Delay {
                buffer: buffer,
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

// TODO: factor out; same function is in `sequencer.rs`
fn sec_to_sample_num(sec: f64, time: &Time) -> u64 {
    (time.sample_rate as f64 * sec) as u64
}

impl Unit for Delay {
    fn proc(&mut self, time: &Time) -> Signal {
        self.buffer.pop_back();
        let sig = self.src.0.lock().unwrap().proc(time);
        self.buffer.push_front(Box::new(sig));
        let dtime = self.time.0.lock().unwrap().proc(time).0;
        let dt = sec_to_sample_num(dtime, time);
        let fb = self.feedback.0.lock().unwrap().proc(time).0;
        let mix = self.mix.0.lock().unwrap().proc(time).0;

        let (mut dl, mut dr) = (0.0, 0.0);
        let mut n = 1;
        while n * dt < self.buffer.len() as u64 {
            let (l, r) = **self.buffer.get((n * dt) as usize).unwrap();
            let fbr = fb.powi(n as i32);
            dl += l * fbr;
            dr += r * fbr;
            n += 1;
        }

        (sig.0 + dl * mix, sig.1 + dr * mix)
    }
}
