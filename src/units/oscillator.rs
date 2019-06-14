use std::sync::{Arc, Mutex};

use super::super::time::Time;

use super::unit::Signal;
use super::unit::Unit;
use super::unit::UType;
use super::unit::Osc;
use super::unit::UnitGraph;

use super::core::Gain;
use super::core::Offset;

pub struct Sine {
    pub init_ph: Arc<Mutex<UnitGraph>>,
    pub ph: f64,
    pub freq: Arc<Mutex<UnitGraph>>,
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
        let ph_diff = time.sample_rate as f64 * std::f64::consts::PI;
        self.ph += self.freq.lock().unwrap().calc(&time).0 / ph_diff;
    }
}

impl Osc for Sine {
    fn set_freq(&mut self, u: Arc<Mutex<UnitGraph>>) {
        self.freq = u;
    }
}

pub struct Tri {
    pub init_ph: Arc<Mutex<UnitGraph>>,
    pub ph: f64,
    pub freq: Arc<Mutex<UnitGraph>>,
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
    fn set_freq(&mut self, u: Arc<Mutex<UnitGraph>>) {
        self.freq = u;
    }
}

pub struct Saw {
    pub init_ph: Arc<Mutex<UnitGraph>>,
    pub ph: f64,
    pub freq: Arc<Mutex<UnitGraph>>,
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
    fn set_freq(&mut self, u: Arc<Mutex<UnitGraph>>) {
        self.freq = u;
    }
}

pub struct Pulse {
    pub init_ph: Arc<Mutex<UnitGraph>>,
    pub ph: f64,
    pub freq: Arc<Mutex<UnitGraph>>,
    pub duty: Arc<Mutex<UnitGraph>>,
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
    fn set_freq(&mut self, u: Arc<Mutex<UnitGraph>>) {
        self.freq = u;
    }
}

pub struct WaveTable {
    pub table: Vec<f64>,
    pub ph: Arc<Mutex<Unit>>,
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
