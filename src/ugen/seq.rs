use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use super::super::mtime::{Time, Pos, PosOps, Measure};
use super::super::event::{Event, Message, Pitch, to_freq};

use super::core::{Signal, UgNode, Value, Slot, Dump, Walk, UG, UGen, Aug, Proc, Osc, ADSR, Eg, Table};

pub struct Trigger {
    eg: Aug,
    egs: Vec<Aug>,
}

impl Trigger {
    pub fn new(eg: Aug, egs: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(
            Box::new(Trigger {
                eg: eg, egs: egs,
            })
        )))
    }
}

impl Walk for Trigger {
    fn walk(&self, f: &mut FnMut(&Aug) -> bool) {
        if f(&self.eg) {
            self.eg.walk(f);
        }
        for eg in &self.egs {
            if f(eg) {
                eg.walk(f);
            }
        }
    }
}

impl Dump for Trigger {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();
        let mut values = Vec::new();

        slots.push(Slot {
            name: "eg".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.eg) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.eg.clone()),
            },
        });

        for eg in self.egs.iter() {
            values.push(match shared_ug.iter().position(|e| *e == *eg) {
                Some(n) => Box::new(Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone())),
                None => Box::new(Value::Ug(eg.clone())),
            });
        }

        UgNode::UgRest("trig".to_string(), slots, values)
    }
}

impl Proc for Trigger {
    fn proc(&mut self, time: &Time) -> Signal {
        for eg in &mut self.egs {
            eg.proc(&time);
        }
        self.eg.proc(&time)
    }
}

impl Eg for Trigger {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
            eg.set_state(state.clone(), eplaced);
        }
        for eg in &self.egs {
            if let UG::Eg(ref mut eg) = &mut eg.0.lock().unwrap().ug {
                eg.set_state(state.clone(), eplaced);
            }
        }
    }
}

pub struct AdsrEg {
    a: Aug,
    d: Aug,
    s: Aug,
    r: Aug,
    state: ADSR,
    eplaced: u64,
}

impl AdsrEg {
    pub fn new(a: Aug, d: Aug, s: Aug, r: Aug) -> Aug {
        Aug::new(UGen::new(UG::Eg(
            Box::new(AdsrEg {
                a: a, d: d, s: s, r: r,
                state: ADSR::None,
                eplaced: 0,
            })
        )))
    }
}

fn sec_to_sample_num(sec: f64, time: &Time) -> u64 {
    (time.sample_rate as f64 * sec) as u64
}

impl Walk for AdsrEg {
    fn walk(&self, f: &mut FnMut(&Aug) -> bool) {
        if f(&self.a) {
            self.a.walk(f);
        }
        if f(&self.d) {
            self.d.walk(f);
        }
        if f(&self.s) {
            self.s.walk(f);
        }
        if f(&self.r) {
            self.r.walk(f);
        }
    }
}

impl Dump for AdsrEg {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            name: "a".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.a) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.a.clone()),
            }
        });
        slots.push(Slot {
            name: "d".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.d) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.d.clone()),
            }
        });
        slots.push(Slot {
            name: "s".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.s) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.s.clone()),
            }
        });
        slots.push(Slot {
            name: "r".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.r) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.r.clone()),
            }
        });

        UgNode::Ug("adsr".to_string(), slots)
    }
}

impl Proc for AdsrEg {
    fn proc(&mut self, time: &Time) -> Signal {
        let a = sec_to_sample_num(self.a.proc(time).0, time);
        let d = sec_to_sample_num(self.d.proc(time).0, time);
        let s = self.s.proc(time).0;
        let r = sec_to_sample_num(self.r.proc(time).0, time);
        let state = &self.state;
        let eplaced = self.eplaced;
        let v;

        match state {
            ADSR::Attack => {
                if eplaced < a {
                    v = self.eplaced as f64 / a as f64;
                } else if eplaced < a + d {
                    v = 1.0 - (1.0 - s) * ((eplaced as f64 - a as f64) / d as f64);
                    self.state = ADSR::Decay;
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            },
            ADSR::Decay => {
                if eplaced < a + d {
                    v = 1.0 - (1.0 - s) * ((eplaced as f64 - a as f64) / d as f64);
                } else if eplaced >= a + d {
                    v = s;
                    self.state = ADSR::Sustin;
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            },
            ADSR::Sustin => {
                v = s;
            },
            ADSR::Release => {
                if eplaced < r {
                    v = s - eplaced as f64 * (s / r as f64);
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            },
            ADSR::None => {
                v = 0.0;
            },
        }
        self.eplaced += 1;
        (v, v)
    }
}

impl Eg for AdsrEg {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        self.state = state;
        self.eplaced = eplaced;
    }
}

// pub struct Seq {
//     pattern: Aug,
//     queue: VecDeque<Box<Event>>,
//     osc: Aug,
//     eg: Aug,
// }

// impl Seq {
//     pub fn new(pat: Aug, osc: Aug, eg: Aug, time: &Time) -> Aug {
//         let mut seq = Seq {
//                 pattern: pat,
//                 queue: VecDeque::new(),
//                 osc: osc,
//                 eg: eg,
//         };
//         seq.fill_queue(&time.pos, &time.measure);
//         Mut::amut(UnitGraph::new(Node::Sig(Mut::amut(seq))))
//     }

//     pub fn fill_queue(&mut self, base: &Pos, measure: &Measure) {
//         let mut pos = base.clone();
//         if let Node::Pat(pat) = &self.pattern.0.lock().unwrap().node {
//             for m in pat.0.lock().unwrap().0.iter() {
//                 match &**m {
//                     Message::Note(pitch, len) => {
//                         match pitch {
//                             Pitch::Pitch(_, _) => {
//                                 self.queue.push_back(Box::new(Event::On(pos.clone(), to_freq(pitch))));
//                                 pos = pos.clone().add(len.clone(), &measure);
//                                 self.queue.push_back(Box::new(Event::Off(pos.clone())));
//                             },
//                             Pitch::Rest => {
//                                 pos = pos.clone().add(len.clone(), &measure);
//                             },
//                         }
//                     },
//                     Message::Loop => {
//                         self.queue.push_back(Box::new(Event::Loop(pos.clone())));
//                     },
//                 }
//             }
//         } else {
//             panic!("not a pattern!!");
//         }
//     }
// }

// impl Walk for Seq {
//     fn walk(&self, f: &mut FnMut(&Aug) -> bool) {
//         if f(&self.pattern) { self.pattern.0.lock().unwrap().walk(f); }
//         if f(&self.osc) { self.osc.0.lock().unwrap().walk(f); }
//         if f(&self.eg) {  self.eg.0.lock().unwrap().walk(f); }
//     }
// }

// impl Dump for Seq {
//     fn dump(&self, shared_ug: &Vec<Aug>, shared_map: &HashMap<usize, String>) -> UgNode {
//         let mut nvec = Vec::new();
//         let mut values = Vec::new();

//         nvec.push("pattern".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.pattern)) {
//             Some(idx) => values.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => values.push(Box::new(self.pattern.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         nvec.push("osc".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.osc)) {
//             Some(idx) => values.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => values.push(Box::new(self.osc.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         nvec.push("eg".to_string());
//         match shared_ug.iter().position(|e| Arc::ptr_eq(e, &self.eg)) {
//             Some(idx) => values.push(Box::new(UgNode::Value(shared_map.get(&idx).unwrap().to_string()))),
//             None => values.push(Box::new(self.eg.0.lock().unwrap().dump(shared_ug, shared_map))),
//         }

//         UgNode::Op("seq".to_string(), nvec, values)
//     }
// }

// impl Proc for Seq {
//     fn proc(&mut self, time: &Time) -> Signal {
//         let (ol, or) = self.osc.0.lock().unwrap().proc(&time);
//         let (el, er) = self.eg.0.lock().unwrap().proc(&time);
//         let mut q = self.queue.iter().peekable();
//         match q.peek() {
//             Some(e) => match &***e {
//                 Event::On(pos, _freq) => if pos <= &time.pos {
//                     if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
//                         if let Node::Osc(osc) = &self.osc.0.lock().unwrap().node {
//                             osc.0.lock().unwrap().set_freq(Mut::amut(UnitGraph::new(Node::Val(freq))));
//                         }
//                         if let Node::Eg(eg) = &self.eg.0.lock().unwrap().node {
//                             eg.0.lock().unwrap().set_state(ADSR::Attack, 0);
//                         }
//                     }
//                 },
//                 Event::Off(pos) => if pos <= &time.pos {
//                     if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
//                         if let Node::Eg(eg) = &self.eg.0.lock().unwrap().node {
//                             eg.0.lock().unwrap().set_state(ADSR::Release, 0);
//                         }
//                     }
//                 },
//                 Event::Loop(pos) => if pos <= &time.pos {
//                     let base = Pos { bar: time.pos.bar, beat: 0, pos: 0.0 };
//                     self.queue.pop_front().unwrap();
//                     self.fill_queue(&base, &time.measure);
//                 },
//             },
//             None => (),
//         }
//         ((ol * el), (or * er))
//     }
// }
