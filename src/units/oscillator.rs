use std::sync::{Arc, Mutex};

use super::super::time::Time;
use super::unit::Value;
use super::unit::Signal;
use super::unit::Unit;

use super::core::Gain;
use super::core::Offset;

pub struct Sine<'a> {
    pub init_ph: &'a Unit<'a>,
    pub ph: f64,
    pub freq: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Sine<'a> {
    fn calc(&self, time: &Time) -> Value {
        let init_ph = self.init_ph.calc(&time).0;
        let v = (init_ph + self.ph).sin();
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64 * std::f64::consts::PI;
    }

    fn set_freq(&mut self, u: &'a Unit<'a>) {
        self.freq = u;
    }
}

pub struct Tri<'a> {
    pub init_ph: &'a Unit<'a>,
    pub ph: f64,
    pub freq: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Tri<'a> {
    fn calc(&self, time: &Time) -> Value {
        let ph = self.init_ph.calc(&time).0 + self.ph;
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
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64;
    }

    fn set_freq(&mut self, u: &Unit<'_>) {
        self.freq = u;
    }
}

pub struct Saw<'a> {
    pub init_ph: &'a Unit<'a>,
    pub ph: f64,
    pub freq: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Saw<'a> {
    fn calc(&self, time: &Time) -> Value {
        let ph = self.init_ph.calc(&time).0 + self.ph;
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
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64;
    }

    fn set_freq(&mut self, u: &Unit<'a>) {
        self.freq = u;
    }
}

pub struct Pulse<'a> {
    pub init_ph: &'a Unit<'a>,
    pub ph: f64,
    pub freq: &'a Unit<'a>,
    pub duty: &'a Unit<'a>,
}

impl<'a> Signal<'a> for Pulse<'a> {
    fn calc(&self, time: &Time) -> Value {
        let ph = self.init_ph.calc(&time).0 + self.ph;
        let duty = self.duty.calc(&time).0;
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
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.duty.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64;
    }

    fn set_freq(&mut self, u: &Unit) {
        self.freq = u;
    }
}

pub struct TablePhase<'a> {
    osc: &'a Unit<'a>,
    root: Unit<'a>,
}

impl<'a> TablePhase<'a> {
    pub fn new(u: &'a Unit) -> Unit<'a> {
        let osc: &'a Unit = u;
        let root = Unit::Unit(Arc::new(Mutex::new(Gain {
            v: 0.5,
            src: Unit::Unit(Arc::new(Mutex::new(Offset {
                v: 1.0,
                src: *u,
            }))),
        })));
        Unit::Unit(Arc::new(Mutex::new(TablePhase {
            root: root,
            osc: osc,
        })))
    }
}

impl<'a> Signal<'a> for TablePhase<'a> {
    fn calc(&self, time: &Time) -> Value {
        self.root.calc(&time)
    }

    fn update(&mut self, time: &Time) {
        self.root.update(&time);
    }

    fn set_freq(&mut self, u: &Unit<'a>) {
        self.osc = u;
    }
}

pub struct WaveTable<'a> {
    pub table: Vec<f64>,
    pub ph: &'a Unit<'a>,
}

fn linear_interpol(v1: f64, v2: f64, r: f64) -> f64 {
    let r = r % 1.0;
    v1 * r + v2 * (1.0 - r)
}

impl<'a> Signal<'a> for WaveTable<'a> {
    fn calc(&self, time: &Time) -> Value {
        let len = self.table.len() as f64;
        let p = self.ph.calc(&time).0 * len;
        let pos1 = (p.floor() % len) as usize;
        let pos2 = (p.ceil() % len) as usize;
        let v = linear_interpol(self.table[pos1], self.table[pos2], p.fract());
        (v, v)
    }

    fn update(&mut self, time: &Time) {
        self.ph.update(&time);
    }

    fn set_freq(&mut self, u: &Unit<'a>) {
        self.ph.set_freq(u);
    }
}
