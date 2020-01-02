use std::collections::VecDeque;

use super::super::event::{to_freq, Event, Message, Pitch};
use super::super::mtime::{Measure, Pos, PosOps, Time};

use super::core::{Aug, Dump, Eg, Proc, Setv, Signal, Slot, UGen, UgNode, Value, Walk, ADSR, UG};

pub struct Trigger {
    eg: Aug,
    egs: Vec<Aug>,
}

impl Trigger {
    pub fn new(eg: Aug, egs: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Eg(Box::new(Trigger { eg: eg, egs: egs }))))
    }
}

impl Walk for Trigger {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
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
            ug: self.eg.clone(),
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

        UgNode::UgRest("trig".to_string(), slots, "src".to_string(), values)
    }
}

impl Setv for Trigger {
    fn setv(&mut self, pname: &str, data: String, shared: &Vec<Aug>) {}
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
        Aug::new(UGen::new(UG::Eg(Box::new(AdsrEg {
            a: a,
            d: d,
            s: s,
            r: r,
            state: ADSR::None,
            eplaced: 0,
        }))))
    }
}

fn sec_to_sample_num(sec: f64, time: &Time) -> u64 {
    (time.sample_rate as f64 * sec) as u64
}

impl Walk for AdsrEg {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
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
            ug: self.a.clone(),
            name: "a".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.a) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.a.clone()),
            },
        });
        slots.push(Slot {
            ug: self.d.clone(),
            name: "d".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.d) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.d.clone()),
            },
        });
        slots.push(Slot {
            ug: self.s.clone(),
            name: "s".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.s) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.s.clone()),
            },
        });
        slots.push(Slot {
            ug: self.r.clone(),
            name: "r".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.r) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.r.clone()),
            },
        });

        UgNode::Ug("adsr".to_string(), slots)
    }
}

impl Setv for AdsrEg {
    fn setv(&mut self, pname: &str, data: String, shared: &Vec<Aug>) {}
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
            }
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
            }
            ADSR::Sustin => {
                v = s;
            }
            ADSR::Release => {
                if eplaced < r {
                    v = s - eplaced as f64 * (s / r as f64);
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            }
            ADSR::None => {
                v = 0.0;
            }
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
    pattern: Aug,
    queue: VecDeque<Box<Event>>,
    osc: Aug,
    eg: Aug,
}

impl Seq {
    pub fn new(pat: Aug, osc: Aug, eg: Aug, time: &Time) -> Aug {
        let mut seq = Seq {
            pattern: pat,
            queue: VecDeque::new(),
            osc: osc,
            eg: eg,
        };
        seq.fill_queue(&time.pos, &time.measure);
        Aug::new(UGen::new(UG::Proc(Box::new(seq))))
    }

    pub fn fill_queue(&mut self, base: &Pos, measure: &Measure) {
        let mut pos = base.clone();
        if let UG::Pat(pat) = &self.pattern.0.lock().unwrap().ug {
            for m in pat.0.lock().unwrap().iter() {
                match &**m {
                    Message::Note(pitch, len) => match pitch {
                        Pitch::Pitch(_, _) => {
                            self.queue
                                .push_back(Box::new(Event::On(pos.clone(), to_freq(pitch))));
                            pos = pos.clone().add(len.clone(), &measure);
                            self.queue.push_back(Box::new(Event::Off(pos.clone())));
                        }
                        Pitch::Kick => {
                            self.queue
                                .push_back(Box::new(Event::Kick(pos.clone())));
                            pos = pos.clone().add(len.clone(), &measure);
                            self.queue.push_back(Box::new(Event::Off(pos.clone())));
                        }
                        Pitch::Rest => {
                            pos = pos.clone().add(len.clone(), &measure);
                        }
                    },
                    Message::Loop => {
                        self.queue.push_back(Box::new(Event::Loop(pos.clone())));
                    }
                }
            }
        } else {
            panic!("not a pattern!!");
        }
    }
}

impl Walk for Seq {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.pattern) {
            self.pattern.walk(f);
        }
        if f(&self.osc) {
            self.osc.walk(f);
        }
        if f(&self.eg) {
            self.eg.walk(f);
        }
    }
}

impl Dump for Seq {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.pattern.clone(),
            name: "pattern".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.pattern) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.pattern.clone()),
            },
        });
        slots.push(Slot {
            ug: self.osc.clone(),
            name: "osc".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.osc) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.osc.clone()),
            },
        });
        slots.push(Slot {
            ug: self.eg.clone(),
            name: "eg".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.eg) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.eg.clone()),
            },
        });

        UgNode::Ug("seq".to_string(), slots)
    }
}

impl Setv for Seq {
    fn setv(&mut self, pname: &str, data: String, shared: &Vec<Aug>) {}
}

impl Proc for Seq {
    fn proc(&mut self, time: &Time) -> Signal {
        let (ol, or) = self.osc.proc(&time);
        let (el, er) = self.eg.proc(&time);
        let mut q = self.queue.iter().peekable();
        match q.peek() {
            Some(e) => match &***e {
                Event::On(pos, _freq) => {
                    if pos <= &time.pos {
                        if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
                            if let UG::Osc(ref mut osc) = &mut self.osc.0.lock().unwrap().ug {
                                osc.set_freq(Aug::new(UGen::new(UG::Val(freq))));
                            }
                            if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
                                eg.set_state(ADSR::Attack, 0);
                            }
                        }
                    }
                }
                Event::Kick(pos) => {
                    if pos <= &time.pos {
                        if let Event::Kick(_pos) = *self.queue.pop_front().unwrap() {
                            if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
                                eg.set_state(ADSR::Attack, 0);
                            }
                        }
                    }
                }
                Event::Off(pos) => {
                    if pos <= &time.pos {
                        if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
                            if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
                                eg.set_state(ADSR::Release, 0);
                            }
                        }
                    }
                }
                Event::Loop(pos) => {
                    if pos <= &time.pos {
                        let base = Pos {
                            bar: time.pos.bar,
                            beat: 0,
                            pos: 0.0,
                        };
                        self.queue.pop_front().unwrap();
                        self.fill_queue(&base, &time.measure);
                    }
                }
            },
            None => (),
        }
        ((ol * el), (or * er))
    }
}
