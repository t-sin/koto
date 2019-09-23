use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::super::mtime::Time;
use super::super::tapirlisp::value::Env;

use super::unit::{Mut, Signal, Walk, UDump, Dump, Unit, Node, UnitGraph, AUnit};


pub struct LPFilter {
    inbuf: [Signal;2],
    outbuf: [Signal;2],
    freq: AUnit,
    q: AUnit,
    src: AUnit,
}

impl LPFilter {
    pub fn new(freq: AUnit, q: AUnit, src: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(LPFilter {
                inbuf: [(0.0, 0.0), (0.0, 0.0)],
                outbuf: [(0.0, 0.0), (0.0, 0.0)],
                freq: freq, q: q, src: src,
            })
        )))
    }
}

impl Walk for LPFilter {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.freq) { self.freq.0.lock().unwrap().walk(f); }
        if f(&self.q) { self.q.0.lock().unwrap().walk(f); }
        if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
    }
}

impl Dump for LPFilter {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut nvec = Vec::new();
        let mut vvec = Vec::new();

        nvec.push("freq".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.freq)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.freq.0.lock().unwrap().dump(shared_vec, shared_map))),
        }

        nvec.push("q".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.q)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.q.0.lock().unwrap().dump(shared_vec, shared_map))),
        }

        nvec.push("src".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op(Mut::amut(self), "lpf".to_string(), nvec, vvec)
    }
}

impl Unit for LPFilter {
    fn proc(&mut self, time: &Time) -> Signal {
        let f = self.freq.0.lock().unwrap().proc(time).0;
        let q = self.q.0.lock().unwrap().proc(time).0;
        let (sl, sr) = self.src.0.lock().unwrap().proc(time);

        let w = (2.0 * std::f64::consts::PI * f) / time.sample_rate as f64;
        let (sw, cw) = (w.sin(), w.cos());
        let a = sw / (2.0 * q);
        let (b0, b1, b2) = ((1.0 - cw) / 2.0, 1.0 - cw, (1.0 - cw) / 2.0);
        let (a0, a1, a2) = (1.0 + a, -2.0 * cw, 1.0 - a);

        let filter = |v, in0, in1, out0, out1| {
            (b0 / a0 * v) + (b1 / a0 * in0) + (b2 / a0 * in1) - (a1 / a0 * out0) - (a2 / a0 * out1)
        };

        let l = filter(sl, self.inbuf[0].0, self.inbuf[1].0, self.outbuf[0].0, self.outbuf[1].0);
        let r = filter(sr, self.inbuf[0].1, self.inbuf[1].1, self.outbuf[0].1, self.outbuf[1].1);

        self.inbuf[1] = self.inbuf[0];
        self.inbuf[0] = (sl, sr);
        self.outbuf[1] = self.outbuf[0];
        self.outbuf[0] = (l, r);

        (l, r)
    }
}

pub struct Delay {
    buffer: VecDeque<Box<Signal>>,
    time: AUnit,
    feedback: AUnit,
    mix: AUnit,
    src: AUnit,
}

impl Delay {
    pub fn new(time: AUnit, feedback: AUnit, mix: AUnit, src: AUnit, env: &Env) -> AUnit {
        let len = (env.time.sample_rate * 2) as usize;
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
        let mut nvec = Vec::new();
        let mut vvec = Vec::new();

        nvec.push("time".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.time)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.time.0.lock().unwrap().dump(shared_vec, shared_map))),
        }

        nvec.push("feedback".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.feedback)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.feedback.0.lock().unwrap().dump(shared_vec, shared_map))),
        }

        nvec.push("mix".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.mix)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.mix.0.lock().unwrap().dump(shared_vec, shared_map))),
        }

        nvec.push("src".to_string());
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
            Some(idx) => vvec.push(Box::new(UDump::Value(shared_map.get(&idx).unwrap().to_string()))),
            None => vvec.push(Box::new(self.src.0.lock().unwrap().dump(shared_vec, shared_map))),
        }

        UDump::Op(Mut::amut(self), "delay".to_string(), nvec, vvec)
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
