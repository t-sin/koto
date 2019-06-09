use std::collections::VecDeque;

use super::super::time::Time;
use super::super::time::Pos;
use super::unit::Value;
use super::unit::Stateful;
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
    fn calc(&self, time: &Time) -> Value {
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
}

impl Stateful for ADSREnvelope {
    fn update(&mut self, time: &Time) {
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

pub struct Seq {
    pattern: Vec<Box<Event>>,
    queue: VecDeque<Box<Event>>,
    osc: Unit,
    eg: Unit,
}

impl Signal for Seq {
    fn calc(&self, time: &Time) -> Value {
        let (ol, or) = self.osc.calc(&time);
        let (el, er) = self.eg.calc(&time);
        ((ol * el), (or, er))
    }
}

impl Stateful for Seq {
    fn update(&mut self, time: &Time) {
        self.osc.update(&time);
        self.osc.update(&time);

        let q = self.queue.iter().peekable();
        match q.peek() {
            Event::On(pos, _freq) => if pos >= time.pos {
                let Event::On(pos, freq) = self.queue.pop_front();
                self.osc.setFreq(freq);
                self.eg.eplaced = 0;
                self.eg.state = ADSR::Attack;
            },
            Event::Off(pos) => if pos >= time.pos {
                let Event::Off(pos) = self.queue.pop_front();
                self.eg.state = ADSR::Release;
            },
            Event::Loop(pos) => if pos >= time.pos {
                let Event::Loop(pos) = self.queue.pop_front();
                self.pattern.for_each(|ev| {
                    self.queue.push_back(match ev {
                        Event::On(pos, freq) => Event::On(pos + (time.bar, 0, 0.0), freq),
                        Event::Off(pos) => Event::Off(pos + (time.bar, 0, 0.0)),
                        Event::Loop(pos) => Event::Loop(pos + (time.bar, 0, 0.0)),
                    });
                });
            }
        }
    }
}
