use super::super::time::Time;
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

        if eplaced < 0 {
            v = 0.0;
        } else {
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
        }
        (v, v)
    }
}

impl Stateful for ADSREnvelope {
    fn update(&mut self, time: &Time) {
        let state = &self.state;
        let eplaced = self.eplaced;

        if eplaced < 0 {
            self.state = ADSR::None;
        } else {
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
        }
        self.eplaced += 1;
    }
}
