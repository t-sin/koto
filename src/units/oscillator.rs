use super::super::time::Time;
use super::unit::Value;
use super::unit::Stateful;
use super::unit::Signal;
use super::unit::Unit;

use super::core::Gain;
use super::core::Offset;

pub trait Osc {
    fn set_freq(&mut self, u: Unit);
}

pub struct Sine {
    pub init_ph: Unit,
    pub ph: f64,
    pub freq: Unit,
}

impl Signal for Sine {
    fn calc(&self, time: &Time) -> Value {
        let init_ph = self.init_ph.calc(&time).0;
        let v = (init_ph + self.ph).sin();
        (v, v)
    }
}

impl Stateful for Sine {
    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64 * std::f64::consts::PI;
    }
}

impl Osc for Sine {
    fn set_freq(&mut self, u: Unit) {
        self.freq = u;
    }
}

pub struct Tri {
    pub init_ph: Unit,
    pub ph: f64,
    pub freq: Unit,
}

impl Signal for Tri {
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
}

impl Stateful for Tri {
    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64;
    }
}

impl Osc for Tri {
    fn set_freq(&mut self, u: Unit) {
        self.freq = u;
    }
}

pub struct Saw {
    pub init_ph: Unit,
    pub ph: f64,
    pub freq: Unit,
}

impl Signal for Saw {
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
}

impl Stateful for Saw {
    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64;
    }
}

impl Osc for Saw {
    fn set_freq(&mut self, u: Unit) {
        self.freq = u;
    }
}

pub struct Pulse {
    pub init_ph: Unit,
    pub ph: f64,
    pub freq: Unit,
    pub duty: Unit,
}

impl Signal for Pulse {
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
}

impl Stateful for Pulse {
    fn update(&mut self, time: &Time) {
        self.init_ph.update(&time);
        self.freq.update(&time);
        self.duty.update(&time);
        self.ph += self.freq.calc(&time).0 / time.sample_rate as f64;
    }
}

impl Osc for Pulse {
    fn set_freq(&mut self, u: Unit) {
        self.freq = u;
    }
}

pub struct TablePhase {
    osc: Unit,
    root: Unit,
}

pub fn make_table_phase(u: Unit) {
    TablePhase {
        root: Unit::Unit(Box::new(Gain {
            v: 0.5,
            src: Unit::Unit(Box::new(Offset {
                v: 1.0,
                src: u,
            })),
        })),
        osc: u,
    }
}

impl Signal for TablePhase {
    fn calc(&self, time: &Time) -> Value {
        self.root.calc(&time)
    }
}

impl Stateful for TablePhase {
    fn update(&mut self, time: &Time) {
        self.root.update(&time);
    }
}

impl Osc for TablePhase {
    fn set_freq(&mut self, u: Unit) {
        self.osc = u;
    }
}

pub struct WaveTable {
    pub table: Vec<f64>,
    pub ph: TablePhase,
}

fn linear_interpol(v1: f64, v2: f64, r: f64) -> f64 {
    let r = r % 1.0;
    v1 * r + v2 * (1.0 - r)
}

impl Signal for WaveTable {
    fn calc(&self, time: &Time) -> Value {
        let len = self.table.len() as f64;
        let p = self.ph.calc(&time).0 * len;
        let pos1 = (p.floor() % len) as usize;
        let pos2 = (p.ceil() % len) as usize;
        let v = linear_interpol(self.table[pos1], self.table[pos2], p.fract());
        (v, v)
    }
}

impl Stateful for WaveTable {
    fn update(&mut self, time: &Time) {
        self.ph.update(&time);
    }
}

impl Osc for WaveTable {
    fn set_freq(&mut self, u: Unit) {
        self.ph.set_freq(u);
    }
}
