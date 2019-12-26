extern crate num;

use std::collections::HashMap;
use std::sync::Arc;

use super::super::mtime::Time;
use super::core::{Signal, UgNode, Slot, Value, Dump, Walk, UG, UGen, Aug, Proc};

pub struct Pan {
    pub v: Aug,
    pub src: Aug,
}

impl Pan {
    pub fn new(v: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Pan { v: v, src: src })
        )))
    }
}

impl Walk for Pan {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.v) { self.v.walk(f); }
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Pan {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            name: "v".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.v) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.v.clone()),
            },
        });
        slots.push(Slot {
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("pan".to_string(), slots)
    }
}

impl Proc for Pan {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        let v = self.v.proc(&time).0;

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
    pub src: Aug,
}

impl Clip {
    pub fn new(min: f64, max: f64, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(
           Box::new(Clip { min: min, max: max, src: src })
        )))
    }
}

impl Walk for Clip {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Clip {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            name: "min".to_string(),
            value: Value::Number(self.min),
        });
        slots.push(Slot {
            name: "max".to_string(),
            value: Value::Number(self.max),
        });
        slots.push(Slot {
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("clip".to_string(), slots)
    }
}

impl Proc for Clip {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        (num::clamp(l, self.min, self.max), num::clamp(r, self.min, self.max))
    }
}

pub struct Offset {
    pub v: f64,
    pub src: Aug,
}

impl Offset {
    pub fn new(v: f64, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Offset { v: v, src: src })
        )))
    }
}

impl Walk for Offset {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Offset {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            name: "v".to_string(),
            value: Value::Number(self.v),
        });
        slots.push(Slot {
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("offset".to_string(), slots)
    }
}

impl Proc for Offset {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        (l + self.v, r + self.v)
    }
}

pub struct Gain {
    pub v: f64,
    pub src: Aug,
}

impl Gain {
    pub fn new(v: f64, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Gain { v: v, src: src })
        )))
    }
}

impl Walk for Gain {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Gain {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            name: "v".to_string(),
            value: Value::Number(self.v),
        });
        slots.push(Slot {
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("gain".to_string(), slots)
    }
}

impl Proc for Gain {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        (l * self.v, r * self.v)
    }
}

pub struct Add {
    pub sources: Vec<Aug>,
}

impl Add {
    pub fn new(sources: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Add { sources: sources })
        )))
    }
}

impl Walk for Add {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.walk(f); }
        }
    }
}

impl Dump for Add {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut values = Vec::new();

        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => values.push(Box::new(Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()))),
                None => values.push(Box::new(Value::Ug(u.clone()))),
            };
        }
        UgNode::UgRest("+".to_string(), Vec::new(), "src".to_string(), values)
    }
}

impl Proc for Add {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.proc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }
}

pub struct Multiply {
    pub sources: Vec<Aug>,
}

impl Multiply {
    pub fn new(sources: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Multiply { sources: sources })
        )))
    }
}

impl Walk for Multiply {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.walk(f); }
        }
    }
}

impl Dump for Multiply {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut values = Vec::new();

        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => values.push(Box::new(Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()))),
                None => values.push(Box::new(Value::Ug(u.clone()))),
            };
        }

        UgNode::UgRest("*".to_string(), Vec::new(), "".to_string(), values)
    }
}

impl Proc for Multiply {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 1.0;
        let mut r = 1.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.proc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }
}

pub struct Out {
    vol: f64,
    sources: Vec<Aug>,
}

impl Out {
    pub fn new(vol: f64, sources: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Out { vol: vol, sources: sources })
        )))
    }
}

impl Walk for Out {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.walk(f); }
        }
    }
}

impl Dump for Out {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();
        let mut values = Vec::new();

        slots.push(Slot {
            name: "v".to_string(),
            value: Value::Number(self.vol)
        });

        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => values.push(Box::new(Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()))),
                None => values.push(Box::new(Value::Ug(u.clone()))),
            }
        }
        UgNode::UgRest("out".to_string(), slots, "src".to_string(), values)
    }
}

impl Proc for Out {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.proc(&time);
            l += l2;
            r += r2;
        }
        (l * self.vol, r * self.vol)
    }
}