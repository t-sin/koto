extern crate num;

use std::collections::HashMap;
use std::sync::Arc;

use super::super::time::Time;

use super::unit::Signal;
use super::unit::{Mut, UDump, Dump, Walk, Unit, Node, UnitGraph, AUnit};

pub struct Pan {
    pub v: AUnit,
    pub src: AUnit,
}

impl Pan {
    pub fn new(v: AUnit, src: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Pan { v: v, src: src })
        )))
    }
}

impl Walk for Pan {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.v) { self.v.0.lock().unwrap().walk(f); }
        if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
    }
}

impl Dump for Pan {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.v)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("pan".to_string(), vec)
    }
}

impl Unit for Pan {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.0.lock().unwrap().proc(&time);
        let v = self.v.0.lock().unwrap().proc(&time).0;

        if v > 0.0 {
            (l * (1.0 - v), r)
        } else if v < 0.0 {
            (l, r * (1.0 - v))
        } else {
            (l, r)
        }
    }
}

pub struct Clip {
    pub min: f64,
    pub max: f64,
    pub src: AUnit,
}

impl Clip {
    pub fn new(min: f64, max: f64, src: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Clip { min: min, max: max, src: src })
        )))
    }
}

impl Walk for Clip {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
    }
}

impl Dump for Clip {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        vec.push(Box::new(UDump::Str(self.min.to_string())));
        vec.push(Box::new(UDump::Str(self.max.to_string())));
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("clip".to_string(), vec)
    }
}

impl Unit for Clip {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.0.lock().unwrap().proc(&time);
        (num::clamp(l, self.min, self.max), num::clamp(r, self.min, self.max))
    }
}

pub struct Offset {
    pub v: f64,
    pub src: AUnit,
}

impl Offset {
    pub fn new(v: f64, src: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Offset { v: v, src: src })
        )))
    }
}

impl Walk for Offset {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
    }
}

impl Dump for Offset {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        vec.push(Box::new(UDump::Str(self.v.to_string())));
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("offset".to_string(), vec)
    }
}

impl Unit for Offset {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.0.lock().unwrap().proc(&time);
        (l + self.v, r + self.v)
    }
}

pub struct Gain {
    pub v: f64,
    pub src: AUnit,
}

impl Gain {
    pub fn new(v: f64, src: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Gain { v: v, src: src })
        )))
    }
}

impl Walk for Gain {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
    }
}

impl Dump for Gain {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        vec.push(Box::new(UDump::Str(self.v.to_string())));
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("gain".to_string(), vec)
    }
}

impl Unit for Gain {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.0.lock().unwrap().proc(&time);
        (l * self.v, r * self.v)
    }
}

pub struct Add {
    pub sources: Vec<AUnit>,
}

impl Add {
    pub fn new(sources: Vec<AUnit>) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Add { sources: sources })
        )))
    }
}

impl Walk for Add {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.0.lock().unwrap().walk(f); }
        }
    }
}

impl Dump for Add {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        for u in self.sources.iter() {
            match shared_vec.iter().position(|e| e == u) {
                Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
                None => vec.push(Box::new(u.0.lock().unwrap().dump(shared_vec, shared_map))),
            };
        }
        UDump::Op("+".to_string(), vec)
    }
}

impl Unit for Add {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.0.lock().unwrap().proc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }
}

pub struct Multiply {
    pub sources: Vec<AUnit>,
}

impl Multiply {
    pub fn new(sources: Vec<AUnit>) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Multiply { sources: sources })
        )))
    }
}

impl Walk for Multiply {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.0.lock().unwrap().walk(f); }
        }
    }
}

impl Dump for Multiply {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        for u in self.sources.iter() {
            match shared_vec.iter().position(|e| e == u) {
                Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
                None => vec.push(Box::new(u.0.lock().unwrap().dump(shared_vec, shared_map))),
            };
        }
        UDump::Op("*".to_string(), vec)
    }
}

impl Unit for Multiply {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 1.0;
        let mut r = 1.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.0.lock().unwrap().proc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }
}
