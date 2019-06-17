use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::super::time::{Time, PosOps};
use super::super::event::Event;
use super::unit::{Signal, AUnit};
use super::unit::{Unit, UType, UnitGraph, ADSR, Eg};

pub struct AdsrEg {
    a: u64,
    d: u64,
    s: f64,
    r: u64,
    state: ADSR,
    eplaced: u64,
}

impl AdsrEg {
    pub fn new(a: u64, d: u64, s: f64, r: u64) -> AUnit {
        Arc::new(Mutex::new(UnitGraph::Unit(UType::Eg(
            Arc::new(Mutex::new(AdsrEg {
                a: a, d: d, s: s, r: r,
                state: ADSR::None,
                eplaced: 0,
            }))
        ))))
    }
}

impl Unit for AdsrEg {
    fn calc(&self, _time: &Time) -> Signal {
        let state = &self.state;
        let eplaced = self.eplaced;
        let v;

        match state {
            ADSR::Attack => {
                if eplaced < self.a {
                    v = self.eplaced as f64 / self.a as f64;
                } else if eplaced < self.a + self.d {
                    v = 1.0 - (1.0 - self.s) * ((eplaced as f64 - self.a as f64) / self.d as f64);
                } else {
                    v = 0.0;
                }
            },
            ADSR::Decay => {
                if eplaced < self.a + self.d {
                    v = 1.0 - (1.0 - self.s) * ((eplaced as f64 - self.a as f64) / self.d as f64);
                } else if eplaced >= self.a + self.d {
                    v = self.s;
                } else {
                    v = 0.0;
                }
            },
            ADSR::Sustin => {
                v = self.s;
            },
            ADSR::Release => {
                if eplaced < self.r {
                    v = self.s - eplaced as f64 * (self.s / self.r as f64);
                } else {
                    v = 0.0;
                }
            },
            ADSR::None => {
                v = 0.0;
            },
        }
        (v, v)
    }

    fn update(&mut self, _time: &Time) {
        let state = &self.state;
        let eplaced = self.eplaced;

        match state {
            ADSR::Attack => {
                if eplaced < self.a {
                    ;
                } else if eplaced < self.a + self.d {
                    self.state = ADSR::Decay;
                } else {
                    self.state = ADSR::None;
                }
            },
            ADSR::Decay => {
                if eplaced < self.a + self.d {
                    ;
                } else if eplaced >= self.a + self.d {
                    self.state = ADSR::Sustin;
                } else {
                    self.state = ADSR::None;
                }
            },
            ADSR::Sustin => {},
            ADSR::Release => {
                if eplaced < self.r {
                } else {
                    self.state = ADSR::None;
                }
            },
            ADSR::None => {},
        }
        self.eplaced += 1;
    }
}

impl Eg for AdsrEg {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        self.state = state;
        self.eplaced = eplaced;
    }
}

pub struct Seq {
    pattern: Vec<Box<Event>>,
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
            UnitGraph::Unit(UType::Sig(
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
    fn calc(&self, time: &Time) -> Signal {
        let (ol, or) = self.osc.lock().unwrap().calc(&time);
        let (el, er) = self.eg.lock().unwrap().calc(&time);
        ((ol * el), (or * er))
    }

    fn update(&mut self, time: &Time) {
        self.osc.lock().unwrap().update(&time);
        self.eg.lock().unwrap().update(&time);

        let mut q = self.queue.iter().peekable();
        match q.peek() {
            Some(e) => match &***e {
                Event::On(pos, _freq) => if pos <= &time.pos {
                    if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
                        if let UnitGraph::Unit(UType::Osc(osc)) = &*self.osc.lock().unwrap() {
                            osc.lock().unwrap().set_freq(Arc::new(Mutex::new(UnitGraph::Value(freq))));
                        }
                        if let UnitGraph::Unit(UType::Eg(eg)) = &*self.eg.lock().unwrap() {
                            eg.lock().unwrap().set_state(ADSR::Attack, 0);
                        }
                    }
                },
                Event::Off(pos) => if pos <= &time.pos {
                    if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
                        if let UnitGraph::Unit(UType::Eg(eg)) = &*self.eg.lock().unwrap() {
                            eg.lock().unwrap().set_state(ADSR::Release, 0);
                        }
                    }
                },
                Event::Loop(pos) => if pos <= &time.pos {
                    if let Event::Loop(_pos) = *self.queue.pop_front().unwrap() {
                        let q = &mut self.queue;
                        self.pattern.iter().for_each(|ev| {
                            q.push_back(match &**ev {
                                Event::On(pos, freq) => Box::new(Event::On(pos.add((time.pos.bar, 0, 0.0), &time), *freq)),
                                Event::Off(pos) => Box::new(Event::Off(pos.add((time.pos.bar, 0, 0.0), &time))),
                                Event::Loop(pos) => Box::new(Event::Loop(pos.add((time.pos.bar, 0, 0.0), &time))),
                            });
                        });
                    }
                },
            },
            None => (),
        }
    }
}
