use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use super::super::mtime::Time;
use super::super::tapirlisp::types::Env;

use super::core::{Signal, UgNode, Value, Slot, Dump, Walk, UG, UGen, Aug, Proc, Osc, Table};


pub struct LPFilter {
    inbuf: [Signal;2],
    outbuf: [Signal;2],
    freq: Aug,
    q: Aug,
    src: Aug,
}

impl LPFilter {
    pub fn new(freq: Aug, q: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(LPFilter {
                inbuf: [(0.0, 0.0), (0.0, 0.0)],
                outbuf: [(0.0, 0.0), (0.0, 0.0)],
                freq: freq, q: q, src: src,
            })
        )))
    }
}

impl Walk for LPFilter {
    fn walk(&self, f: &mut FnMut(&Aug) -> bool) {
        if f(&self.freq) {
            self.freq.walk(f);
        }
        if f(&self.q) {
            self.q.walk(f);
        }
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for LPFilter {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            name: "freq".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.freq) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.freq.clone()),
            },
        });
        slots.push(Slot {
            name: "q".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.q) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.q.clone()),
            },
        });
        slots.push(Slot {
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("lpf".to_string(), slots)
    }
}

impl Proc for LPFilter {
    fn proc(&mut self, time: &Time) -> Signal {
        let f = self.freq.proc(time).0;
        let q = self.q.proc(time).0;
        let (sl, sr) = self.src.proc(time);

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

// pub struct Delay {
//     buffer: VecDeque<Box<Signal>>,
//     time: Aug,
//     feedback: Aug,
//     mix: Aug,
//     src: Aug,
// }

// impl Delay {
//     pub fn new(time: Aug, feedback: Aug, mix: Aug, src: Aug, env: &Env) -> Aug {
//         let len = (env.time.sample_rate * 2) as usize;
//         let mut buffer = VecDeque::with_capacity(len);
//         for _n in 0..len {
//             buffer.push_back(Box::new((0.0, 0.0)));
//         }
//         Mut::amut(UnitGraph::new(Node::Sig(
//             Mut::amut(Delay {
//                 buffer: buffer,
//                 time: time,
//                 feedback: feedback,
//                 mix: mix,
//                 src: src
//             })
//         )))
//     }
// }

// impl Walk for Delay {
//     fn walk(&self, f: &mut FnMut(&Aug) -> bool) {
//         if f(&self.time) { self.time.0.lock().unwrap().walk(f); }
//         if f(&self.feedback) { self.feedback.0.lock().unwrap().walk(f); }
//         if f(&self.mix) { self.mix.0.lock().unwrap().walk(f); }
//         if f(&self.src) { self.src.0.lock().unwrap().walk(f); }
//     }
// }

// impl Dump for Delay {
//     fn dump(&self, shared_ug: &Vec<Aug>, shared_map: &HashMap<usize, String>) -> UgNode {
//         let mut nvec = Vec::new();
//         let mut vvec = Vec::new();

//         nvec.push("time".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.time)) {
//             Some(idx) => vvec.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => vvec.push(Box::new(self.time.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         nvec.push("feedback".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.feedback)) {
//             Some(idx) => vvec.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => vvec.push(Box::new(self.feedback.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         nvec.push("mix".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.mix)) {
//             Some(idx) => vvec.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => vvec.push(Box::new(self.mix.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         nvec.push("src".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.src)) {
//             Some(idx) => vvec.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => vvec.push(Box::new(self.src.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         UgNode::Op("delay".to_string(), nvec, vvec)
//     }
// }

// // TODO: factor out; same function is in `sequencer.rs`
// fn sec_to_sample_num(sec: f64, time: &Time) -> u64 {
//     (time.sample_rate as f64 * sec) as u64
// }

// impl Proc for Delay {
//     fn proc(&mut self, time: &Time) -> Signal {
//         self.buffer.pop_back();
//         let sig = self.src.0.lock().unwrap().proc(time);
//         self.buffer.push_front(Box::new(sig));
//         let dtime = self.time.0.lock().unwrap().proc(time).0;
//         let dt = sec_to_sample_num(dtime, time);
//         let fb = self.feedback.0.lock().unwrap().proc(time).0;
//         let mix = self.mix.0.lock().unwrap().proc(time).0;

//         let (mut dl, mut dr) = (0.0, 0.0);
//         let mut n = 1;
//         while n * dt < self.buffer.len() as u64 {
//             let (l, r) = **self.buffer.get((n * dt) as usize).unwrap();
//             let fbr = fb.powi(n as i32);
//             dl += l * fbr;
//             dr += r * fbr;
//             n += 1;
//         }

//         (sig.0 + dl * mix, sig.1 + dr * mix)
//     }
// }
