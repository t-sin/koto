use std::collections::VecDeque;

use super::super::time::Time;
use super::super::time::Pos;
use super::super::time::PosOps;

use super::unit::Value;
use super::unit::Signal;
use super::unit::Unit;

pub enum ADSR {
    Attack,
    Decay,
    Sustin,
    Release,
    None,
}

pub struct ADSREnvelope {
    a: u64,
    d: u64,
    s: u64,
    r: u64,
    state: ADSR,
    eplaced: u64,
}

impl Signal for ADSREnvelope {
    fn calc(&self, _time: &Time) -> Value {
        let state = &self.state;
        let eplaced = self.eplaced;
        let v;

        match state {
            ADSR::Attack => {
                if eplaced < self.a {
                    v = self.eplaced as f64 / self.a as f64;
                } else if eplaced < self.a + self.d {
                    v = 1.0 - (1.0 - self.s as f64) * ((eplaced as f64 - self.a as f64) / self.d as f64);
                } else {
                    v = 0.0;
                }
            },
            ADSR::Decay => {
                if eplaced < self.a + self.d {
                    v = 1.0 - (1.0 - self.s as f64) * ((eplaced as f64 - self.a as f64) / self.d as f64);
                } else if eplaced >= self.a + self.d {
                    v = self.s as f64;
                } else {
                    v = 0.0;
                }
            },
            ADSR::Sustin => {
                v = self.s as f64;
            },
            ADSR::Release => {
                if eplaced < self.r {
                    v = self.s as f64 - eplaced as f64 * (self.s as f64 / self.r as f64);
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

type Freq = f64;

fn to_freq(note: u32) -> Freq {
    440.0 * ((note - 69) as f64 / 12.0).exp2()
}

pub enum Event {
    On(Pos, Freq),
    Off(Pos),
    Loop(Pos),
}

pub struct Seq<'a> {
    pattern: Vec<Box<Event>>,
    queue: VecDeque<Box<Event>>,
    osc: Arc<Mutex<Unit>>,
    eg: ADSREnvelope,
}

impl Signal for Seq {
    fn calc(&self, time: &Time) -> Value {
        let (ol, or) = self.osc.lock().unwrap().calc(&time);
        let (el, er) = self.eg.calc(&time);
        ((ol * el), (or * er))
    }

    fn update(&mut self, time: &Time) {
        self.osc.lock().unwrap().update(&time);

        let q = self.queue.iter().peekable();
        match q.peek() {
            Some(e) => match ***e {
                Event::On(pos, _freq) => if pos >= time.pos {
                    if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
                        self.osc.set_freq(&Unit::Value(freq));
                        self.eg.eplaced = 0;
                        self.eg.state = ADSR::Attack;
                    }
                },
                Event::Off(pos) => if pos >= time.pos {
                    if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
                        self.eg.state = ADSR::Release;
                    }
                },
                Event::Loop(pos) => if pos >= time.pos {
                    if let Event::Loop(_pos) = *self.queue.pop_front().unwrap() {
                        self.pattern.iter().for_each(|ev| {
                            self.queue.push_back(match **ev {
                                Event::On(pos, freq) => Box::new(Event::On(pos.add((time.pos.bar, 0, 0.0), &time), freq)),
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
