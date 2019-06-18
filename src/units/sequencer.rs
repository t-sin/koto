use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::super::time::{Time, PosOps};
use super::super::event::Event;

use super::unit::{Signal, AUnit};
use super::unit::{Dump, Unit, Node, UnitGraph, ADSR, Eg, Pattern};

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
        Arc::new(Mutex::new(UnitGraph::new(Node::Eg(
            Arc::new(Mutex::new(AdsrEg {
                a: a, d: d, s: s, r: r,
                state: ADSR::None,
                eplaced: 0,
            }))
        ))))
    }
}

fn sec_to_sample_num(sec: f64, time: &Time) -> u64 {
    (time.sample_rate as f64 * sec) as u64
}

impl Unit for AdsrEg {
    fn proc(&mut self, time: &Time) -> Signal {
        let a = sec_to_sample_num(self.a.lock().unwrap().proc(time).0, time);
        let d = sec_to_sample_num(self.d.lock().unwrap().proc(time).0, time);
        let s = self.s.lock().unwrap().proc(time).0;
        let r = sec_to_sample_num(self.r.lock().unwrap().proc(time).0, time);
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

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        vec.push(Box::new(self.a.lock().unwrap().dump()));
        vec.push(Box::new(self.d.lock().unwrap().dump()));
        vec.push(Box::new(self.s.lock().unwrap().dump()));
        vec.push(Box::new(self.r.lock().unwrap().dump()));
        Dump::Op("adsr".to_string(), vec)
    }
}

impl Eg for AdsrEg {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        self.state = state;
        self.eplaced = eplaced;
    }
}

pub struct Seq {
    pattern: Pattern,
    queue: VecDeque<Box<Event>>,
    osc: AUnit,
    eg: AUnit,
}

impl Seq {
    pub fn new(pat: Vec<Box<Event>>, osc: AUnit, eg: AUnit) -> AUnit {
        let mut queue: VecDeque<Box<Event>> = VecDeque::new();
        for e in pat.as_slice().iter() {
            queue.push_back(Box::new(*e.clone()))
        }
        Arc::new(Mutex::new(
            UnitGraph::new(Node::Sig(
                Arc::new(Mutex::new(
                    Seq {
                        pattern: pat,
                        queue: queue,
                        osc: osc,
                        eg: eg,
                    }
                ))
            ))
        ))
    }
}

impl Unit for Seq {
    fn proc(&mut self, time: &Time) -> Signal {
        let (ol, or) = self.osc.lock().unwrap().proc(&time);
        let (el, er) = self.eg.lock().unwrap().proc(&time);
        let mut q = self.queue.iter().peekable();
        match q.peek() {
            Some(e) => match &***e {
                Event::On(pos, _freq) => if pos <= &time.pos {
                    if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
                        if let Node::Osc(osc) = &self.osc.lock().unwrap().node {
                            osc.lock().unwrap().set_freq(
                                Arc::new(Mutex::new(UnitGraph::new(Node::Val(freq))))
                            );
                        }
                        if let Node::Eg(eg) = &self.eg.lock().unwrap().node {
                            eg.lock().unwrap().set_state(ADSR::Attack, 0);
                        }
                    }
                },
                Event::Off(pos) => if pos <= &time.pos {
                    if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
                        if let Node::Eg(eg) = &self.eg.lock().unwrap().node {
                            eg.lock().unwrap().set_state(ADSR::Release, 0);
                        }
                    }
                },
                Event::Loop(pos) => if pos <= &time.pos {
                    if let Event::Loop(_pos) = *self.queue.pop_front().unwrap() {
                        let q = &mut self.queue;
                        self.pattern.iter().for_each(|ev| {
                            q.push_back(match &**ev {
                                Event::On(pos, freq) => {
                                    Box::new(Event::On(pos.add((time.pos.bar, 0, 0.0), &time.measure), *freq))
                                },
                                Event::Off(pos) => {
                                    Box::new(Event::Off(pos.add((time.pos.bar, 0, 0.0), &time.measure)))
                                },
                                Event::Loop(pos) => {
                                    Box::new(Event::Loop(pos.add((time.pos.bar, 0, 0.0), &time.measure)))
                                },
                            });
                        });
                    }
                },
            },
            None => (),
        }
        ((ol * el), (or * er))
    }

    fn dump(&self) -> Dump {
        let mut vec = Vec::new();
        vec.push(Box::new(self.pattern.dump()));
        vec.push(Box::new(self.osc.lock().unwrap().dump()));
        vec.push(Box::new(self.eg.lock().unwrap().dump()));
        Dump::Op("seq".to_string(), vec)
    }
}
