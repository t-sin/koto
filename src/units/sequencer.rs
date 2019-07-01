use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use super::super::time::{Time, Pos, PosOps, Measure};
use super::super::event::{Event, Message, Pitch, to_freq};

use super::unit::{Signal, Mut, AUnit};
use super::unit::{Walk, UDump, Dump, Unit, Node, UnitGraph, ADSR, Eg};

pub struct AdsrEg {
    a: AUnit,
    d: AUnit,
    s: AUnit,
    r: AUnit,
    state: ADSR,
    eplaced: u64,
}

impl AdsrEg {
    pub fn new(a: AUnit, d: AUnit, s: AUnit, r: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Eg(
            Mut::amut(AdsrEg {
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
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.a) { self.a.0.lock().unwrap().walk(f); }
        if f(&self.d) { self.d.0.lock().unwrap().walk(f); }
        if f(&self.s) { self.s.0.lock().unwrap().walk(f); }
        if f(&self.r) { self.r.0.lock().unwrap().walk(f); }
    }
}

impl Dump for AdsrEg {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.a)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.a.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.d)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.d.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.s)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.s.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.r)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.r.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("adsr".to_string(), vec)
    }
}

impl Unit for AdsrEg {
    fn proc(&mut self, time: &Time) -> Signal {
        let a = sec_to_sample_num(self.a.0.lock().unwrap().proc(time).0, time);
        let d = sec_to_sample_num(self.d.0.lock().unwrap().proc(time).0, time);
        let s = self.s.0.lock().unwrap().proc(time).0;
        let r = sec_to_sample_num(self.r.0.lock().unwrap().proc(time).0, time);
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

pub struct Seq {
    pattern: AUnit,
    queue: VecDeque<Box<Event>>,
    osc: AUnit,
    eg: AUnit,
}

impl Seq {
    pub fn new(pat: AUnit, osc: AUnit, eg: AUnit, time: &Time) -> AUnit {
        let mut seq = Seq {
                pattern: pat,
                queue: VecDeque::new(),
                osc: osc,
                eg: eg,
        };
        seq.fill_queue(&time.pos, &time.measure);
        Mut::amut(UnitGraph::new(Node::Sig(Mut::amut(seq))))
    }

    pub fn fill_queue(&mut self, base: &Pos, measure: &Measure) {
        let mut pos = base.clone();
        if let Node::Pat(pat) = &self.pattern.0.lock().unwrap().node {
            for m in pat.0.lock().unwrap().0.iter() {
                match &**m {
                    Message::Note(pitch, len) => {
                        match pitch {
                            Pitch::Pitch(_, _) => {
                                self.queue.push_back(Box::new(Event::On(pos.clone(), to_freq(pitch))));
                                pos = pos.clone().add(len.clone(), &measure);
                                self.queue.push_back(Box::new(Event::Off(pos.clone())));
                            },
                            Pitch::Rest => {
                                pos = pos.clone().add(len.clone(), &measure);
                            },
                        }
                    },
                    Message::Loop => {
                        self.queue.push_back(Box::new(Event::Loop(pos.clone())));
                    },
                }
            }
        } else {
            panic!("not a pattern!!");
        }
    }
}

impl Walk for Seq {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.pattern) { self.pattern.0.lock().unwrap().walk(f); }
        if f(&self.osc) { self.osc.0.lock().unwrap().walk(f); }
        if f(&self.eg) {  self.eg.0.lock().unwrap().walk(f); }
    }
}

impl Dump for Seq {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.pattern)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.pattern.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.osc)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.osc.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.eg)) {
            Some(idx) => vec.push(Box::new(UDump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.eg.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        UDump::Op("seq".to_string(), vec)
    }
}

impl Unit for Seq {
    fn proc(&mut self, time: &Time) -> Signal {
        let (ol, or) = self.osc.0.lock().unwrap().proc(&time);
        let (el, er) = self.eg.0.lock().unwrap().proc(&time);
        let mut q = self.queue.iter().peekable();
        match q.peek() {
            Some(e) => match &***e {
                Event::On(pos, _freq) => if pos <= &time.pos {
                    if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
                        if let Node::Osc(osc) = &self.osc.0.lock().unwrap().node {
                            osc.0.lock().unwrap().set_freq(Mut::amut(UnitGraph::new(Node::Val(freq))));
                        }
                        if let Node::Eg(eg) = &self.eg.0.lock().unwrap().node {
                            eg.0.lock().unwrap().set_state(ADSR::Attack, 0);
                        }
                    }
                },
                Event::Off(pos) => if pos <= &time.pos {
                    if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
                        if let Node::Eg(eg) = &self.eg.0.lock().unwrap().node {
                            eg.0.lock().unwrap().set_state(ADSR::Release, 0);
                        }
                    }
                },
                Event::Loop(pos) => if pos <= &time.pos {
                    let base = Pos { bar: time.pos.bar, beat: 0, pos: 0.0 };
                    self.queue.pop_front().unwrap();
                    self.fill_queue(&base, &time.measure);
                },
            },
            None => (),
        }
        ((ol * el), (or * er))
    }
}
