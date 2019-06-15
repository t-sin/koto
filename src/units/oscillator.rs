use std::sync::{Arc, Mutex};

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use super::super::time::Time;
use super::super::time::Clock;

use super::unit::{Signal, AUnit};
use super::unit::{Unit, UType, Osc, UnitGraph};

use super::core::{Gain, Offset};

pub struct Rand {
    rng: SmallRng,
    v: f64,
}

impl Rand {
    pub fn new(seed: u64) -> AUnit {
        Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Osc(
                Arc::new(Mutex::new(Rand {
                    rng: SmallRng::seed_from_u64(seed),
                    v: 0.15,
                }))
            ))
        ))
    }
}

impl Unit for Rand {
    fn calc(&self, _time: &Time) -> Signal {
        (self.v, self.v)
    }
    fn update(&mut self, _time: &Time) {
        self.v = self.rng.gen();
    }
}

impl Osc for Rand {
    fn set_freq(&mut self, _u: AUnit) {}
}

pub struct Sine {
    pub init_ph: AUnit,
    pub ph: f64,
    pub freq: AUnit,
}

impl Unit for Sine {
    fn calc(&self, time: &Time) -> Signal {
        let init_ph = self.init_ph.lock().unwrap().calc(&time).0;
        let v = (init_ph + self.ph).sin();
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.init_ph.lock().unwrap().update(&time);
        self.freq.lock().unwrap().update(&time);
        let ph_diff = time.sample_rate as f64 / std::f64::consts::PI;
        self.ph += self.freq.lock().unwrap().calc(&time).0 / ph_diff;
    }
}

impl Osc for Sine {
    fn set_freq(&mut self, u: AUnit) {
        self.freq = u;
    }
}

pub struct Tri {
    pub init_ph: AUnit,
    pub ph: f64,
    pub freq: AUnit,
}

impl Unit for Tri {
    fn calc(&self, time: &Time) -> Signal {
        let ph = self.init_ph.lock().unwrap().calc(&time).0 + self.ph;
        let x = ph % 1.0;
        let v;
        if x >= 3.0 / 4.0 {
            v = 4.0 * x - 4.0;
        } else if x >= 1.0 / 4.0 && x < 3.0 / 4.0 {
            v = -4.0 * x + 2.0;
        } else {
            v = 4.0 * x;
        }
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.init_ph.lock().unwrap().update(&time);
        self.freq.lock().unwrap().update(&time);
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.lock().unwrap().calc(&time).0 / ph_diff;
    }
}

impl Osc for Tri {
    fn set_freq(&mut self, u: AUnit) {
        self.freq = u;
    }
}

pub struct Saw {
    pub init_ph: AUnit,
    pub ph: f64,
    pub freq: AUnit,
}

impl Unit for Saw {
    fn calc(&self, time: &Time) -> Signal {
        let ph = self.init_ph.lock().unwrap().calc(&time).0 + self.ph;
        let x = ph % 1.0;
        let v;
        if x >= 1.0 / 2.0 {
            v = 2.0 * x - 2.0;
        } else {
            v = 2.0 * x;
        }
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.init_ph.lock().unwrap().update(&time);
        self.freq.lock().unwrap().update(&time);
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.lock().unwrap().calc(&time).0 / ph_diff;
    }
}

impl Osc for Saw {
    fn set_freq(&mut self, u: AUnit) {
        self.freq = u;
    }
}

pub struct Pulse {
    pub init_ph: AUnit,
    pub ph: f64,
    pub freq: AUnit,
    pub duty: AUnit,
}

impl Unit for Pulse {
    fn calc(&self, time: &Time) -> Signal {
        let ph = self.init_ph.lock().unwrap().calc(&time).0 + self.ph;
        let duty = self.duty.lock().unwrap().calc(&time).0;
        let x = ph % 1.0;
        let v;
        if x < duty {
            v = 1.0;
        } else {
            v = -1.0;
        }
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.init_ph.lock().unwrap().update(&time);
        self.freq.lock().unwrap().update(&time);
        self.duty.lock().unwrap().update(&time);
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.lock().unwrap().calc(&time).0 / ph_diff;
    }
}

impl Osc for Pulse {
    fn set_freq(&mut self, u: AUnit) {
        self.freq = u;
    }
}

pub struct Phase {
    pub root: AUnit,
    pub osc: AUnit,
}

impl Phase {
    pub fn new(u: AUnit) -> AUnit {
        Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Sig(
                Arc::new(Mutex::new(Phase {
                    root: Arc::new(Mutex::new(
                        UnitGraph::Unit(UType::Sig(Arc::new(Mutex::new(Offset {
                            v: 1.0,
                            src: Arc::new(Mutex::new(
                                UnitGraph::Unit(UType::Sig(
                                    Arc::new(Mutex::new(Gain {
                                        v: 0.5,
                                        src: u.clone(),
                                    }
                                ))))
                            )),
                        }
                    ))))
                )),
                osc: u.clone(),
            }))
        ))))
    }
}

impl Unit for Phase {
    fn calc(&self, time: &Time) -> Signal {
        self.root.lock().unwrap().calc(time)
    }
    fn update(&mut self, time: &Time) {
        self.root.lock().unwrap().update(time);
    }
}

impl Osc for Phase {
    fn set_freq(&mut self, freq: AUnit) {
        if let UnitGraph::Unit(UType::Osc(osc)) = &*self.osc.clone().lock().unwrap() {
            osc.lock().unwrap().set_freq(freq);
        } else {
            self.osc = freq;
        }
    }
}

pub struct WaveTable {
    pub table: Vec<f64>,
    pub ph: AUnit,
}

impl WaveTable {
    pub fn new(wave: AUnit, ph: AUnit) -> AUnit {
        let mut table = Vec::new();
        let table_len = 256;
        let mut time = Time::new(table_len / 2, 120.0);
        for _i in 0..table_len {
            let v = wave.lock().unwrap().calc(&time).0;
            table.push(v);
            wave.lock().unwrap().update(&time);
            time.inc();
        }
        Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Osc(
                Arc::new(Mutex::new(
                    WaveTable {
                        table: table,
                        ph: ph,
                    }
                ))
            ))
        ))
    }
}

fn linear_interpol(v1: f64, v2: f64, r: f64) -> f64 {
    let r = r % 1.0;
    v1 * r + v2 * (1.0 - r)
}

impl Unit for WaveTable {
    fn calc(&self, time: &Time) -> Signal {
        let len = self.table.len() as f64;
        let p = self.ph.lock().unwrap().calc(&time).0 * len;
        let pos1 = (p.floor() % len) as usize;
        let pos2 = (p.ceil() % len) as usize;
        let v = linear_interpol(self.table[pos1], self.table[pos2], p.fract());
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.ph.lock().unwrap().update(&time);
    }
}

impl Osc for WaveTable {
    fn set_freq(&mut self, freq: AUnit) {
        if let UnitGraph::Unit(UType::Osc(osc)) = &*self.ph.clone().lock().unwrap() {
            osc.lock().unwrap().set_freq(freq);
        }
    }
}
