extern crate num;

use std::collections::HashMap;
use std::sync::Arc;

use super::super::mtime::Time;
use super::core::{Signal, Param, Dump, Walk, UG, UGen, Aug, Proc};

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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        if f(&self.v) { self.v.walk(f); }
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Pan {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("v".to_string());
        match shared_ug.iter().position(|e| *e == self.v) {
            Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
            None => pvals.push(Box::new(self.v.dump(shared_ug))),
        }

        pnames.push("src".to_string());
        match shared_ug.iter().position(|e| *e == self.src) {
            Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
            None => pvals.push(Box::new(self.src.dump(shared_ug))),
        }
        Param::Ug("pan".to_string(), pnames, pvals)
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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Clip {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("min".to_string());
        pvals.push(Box::new(Param::Value(self.min)));

        pnames.push("max".to_string());
        pvals.push(Box::new(Param::Value(self.max)));

        pnames.push("src".to_string());
        match shared_ug.iter().position(|e| *e == self.src) {
            Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
            None => pvals.push(Box::new(self.src.dump(shared_ug))),
        }
        Param::Ug("clip".to_string(), pnames, pvals)
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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Offset {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("v".to_string());
        pvals.push(Box::new(Param::Value(self.v)));

        pnames.push("src".to_string());
        match shared_ug.iter().position(|e| *e == self.src) {
            Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
            None => pvals.push(Box::new(self.src.dump(shared_ug))),
        }
        Param::Ug("offset".to_string(), pnames, pvals)
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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        if f(&self.src) { self.src.walk(f); }
    }
}

impl Dump for Gain {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("v".to_string());
        pvals.push(Box::new(Param::Value(self.v)));

        pnames.push("src".to_string());
        match shared_ug.iter().position(|e| *e == self.src) {
            Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
            None => pvals.push(Box::new(self.src.dump(shared_ug))),
        }
        Param::Ug("gain".to_string(), pnames, pvals)
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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.walk(f); }
        }
    }
}

impl Dump for Add {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("sources".to_string());
        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
                None => pvals.push(Box::new(u.dump(shared_ug))),
            };
        }
        Param::Ug("+".to_string(), pnames, pvals)
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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.walk(f); }
        }
    }
}

impl Dump for Multiply {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("sources".to_string());
        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
                None => pvals.push(Box::new(u.dump(shared_ug))),
            };
        }
        Param::Ug("*".to_string(), pnames, pvals)
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
    fn walk(&self, f: &mut dyn Fn(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) { s.walk(f); }
        }
    }
}

impl Dump for Out {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Param {
        let mut pnames = Vec::new();
        let mut pvals = Vec::new();

        pnames.push("v".to_string());
        pvals.push(Box::new(Param::Value(self.vol)));

        pnames.push("sources".to_string());
        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => pvals.push(Box::new(Param::Shared(shared_ug.iter().nth(n).unwrap().clone()))),
                None => pvals.push(Box::new(u.dump(shared_ug))),
            }
        }
        Param::UgRest("out".to_string(), pnames, pvals, Vec::new())
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
