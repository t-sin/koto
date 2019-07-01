use std::collections::HashMap;
use std::sync::Arc;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use super::super::time::{Pos, Time};
use super::super::time::Clock;

use super::unit::{Signal, Mut, AUnit};
use super::unit::{Walk, Dump, Unit, Node, UnitGraph, Osc, Table};

use super::core::{Clip, Gain, Offset};

pub struct Rand {
    rng: SmallRng,
    v: f64,
}

impl Rand {
    pub fn new(seed: u64) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(Rand {
                rng: SmallRng::seed_from_u64(seed),
                v: 0.15,
            })
        )))
    }
}

impl Walk for Rand {
    fn walk(&self, _f: &mut FnMut(&AUnit) -> bool) {}
}

impl Unit for Rand {
    fn proc(&mut self, _time: &Time) -> Signal {
        self.v = self.rng.gen();
        (self.v, self.v)
    }

    fn dump(&self, _shared_vec: &Vec<AUnit>, _shared_map: &HashMap<usize, String>) -> Dump {
        Dump::Op("rand".to_string(), vec![Box::new(Dump::Str(self.v.to_string()))])
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

impl Sine {
    pub fn new(init_ph: AUnit, freq: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(Sine { init_ph: init_ph, ph: 0.0, freq: freq })
        )))
    }
}

impl Walk for Sine {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.0.lock().unwrap().walk(f);
        }
        if f(&self.freq){
            self.freq.0.lock().unwrap().walk(f);
        }
    }
}

impl Unit for Sine {
    fn proc(&mut self, time: &Time) -> Signal {
        let init_ph = self.init_ph.0.lock().unwrap().proc(&time).0;
        let v = (init_ph + self.ph).sin();
        let ph_diff = time.sample_rate as f64 / std::f64::consts::PI;
        self.ph += self.freq.0.lock().unwrap().proc(&time).0 / ph_diff;

        (v, v)
    }

    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> Dump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.init_ph)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.init_ph.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.freq)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.freq.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        Dump::Op("sine".to_string(), vec)
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

impl Tri {
    pub fn new(init_ph: AUnit, freq: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(Tri { init_ph: init_ph, ph: 0.0, freq: freq })
        )))
    }
}

impl Walk for Tri {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.0.lock().unwrap().walk(f);
        }
        if f(&self.freq) {
            self.freq.0.lock().unwrap().walk(f);
        }
    }
}

impl Unit for Tri {
    fn proc(&mut self, time: &Time) -> Signal {
        let ph = self.init_ph.0.lock().unwrap().proc(&time).0 + self.ph;

        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.0.lock().unwrap().proc(&time).0 / ph_diff;

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

    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> Dump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.init_ph)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.init_ph.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.freq)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.freq.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        Dump::Op("tri".to_string(), vec)
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

impl Saw {
    pub fn new(init_ph: AUnit, freq: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(Saw { init_ph: init_ph, ph: 0.0, freq: freq })
        )))
    }
}

impl Walk for Saw {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.0.lock().unwrap().walk(f);
        }
        if f(&self.freq) {
            self.freq.0.lock().unwrap().walk(f);
        }
    }
}

impl Unit for Saw {
    fn proc(&mut self, time: &Time) -> Signal {
        let ph = self.init_ph.0.lock().unwrap().proc(&time).0 + self.ph;
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.0.lock().unwrap().proc(&time).0 / ph_diff;

        let x = ph % 1.0;
        let v;
        if x >= 1.0 / 2.0 {
            v = 2.0 * x - 2.0;
        } else {
            v = 2.0 * x;
        }
        (v, v)
    }

    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> Dump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.init_ph)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.init_ph.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.freq)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.freq.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        Dump::Op("saw".to_string(), vec)
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

impl Pulse {
    pub fn new(init_ph: AUnit, freq: AUnit, duty: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(Pulse { init_ph: init_ph, ph: 0.0, freq: freq, duty: duty})
        )))
    }
}

impl Walk for Pulse {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.init_ph) { self.init_ph.0.lock().unwrap().walk(f); }
        if f(&self.freq) { self.freq.0.lock().unwrap().walk(f); }
        if f(&self.duty) { self.duty.0.lock().unwrap().walk(f); }
    }
}

impl Unit for Pulse {
    fn proc(&mut self, time: &Time) -> Signal {
        let ph = self.init_ph.0.lock().unwrap().proc(&time).0 + self.ph;
        let duty = self.duty.0.lock().unwrap().proc(&time).0;
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.0.lock().unwrap().proc(&time).0 / ph_diff;

        let x = ph % 1.0;
        let v;
        if x < duty {
            v = 1.0;
        } else {
            v = -1.0;
        }
        (v, v)
    }

    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> Dump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.init_ph)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.init_ph.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.freq)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.freq.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.duty)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.duty.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        Dump::Op("pulse".to_string(), vec)
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
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(Phase {
                root: Mut::amut(UnitGraph::new(Node::Sig(Mut::amut(Offset {
                    v: 1.0,
                    src: Mut::amut(UnitGraph::new(Node::Sig(
                        Mut::amut(Gain {
                            v: 0.5,
                            src: Mut::amut(UnitGraph::new(Node::Sig(
                                Mut::amut(Clip {
                                    min: 0.0, max: 1.0, src: u.clone(),
                                })
                            ))),
                        })
                    ))),
                })))),
                osc: u.clone(),
            })
        )))
    }
}

impl Walk for Phase {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.osc) { self.osc.0.lock().unwrap().walk(f); }
    }
}

impl Unit for Phase {
    fn proc(&mut self, time: &Time) -> Signal {
        let v = self.root.0.lock().unwrap().proc(time);
        v
    }

    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> Dump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.osc)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.osc.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        Dump::Op("phase".to_string(), vec)
    }
}

impl Osc for Phase {
    fn set_freq(&mut self, freq: AUnit) {
        if let Node::Osc(osc) = &self.osc.clone().0.lock().unwrap().node {
            osc.0.lock().unwrap().set_freq(freq);
        } else {
            self.osc = freq;
        }
    }
}

pub struct WaveTable {
    pub table: AUnit,
    pub ph: AUnit,
}

impl WaveTable {
    pub fn from_osc(osc: AUnit, ph: AUnit, time: &Time) -> AUnit {
        let mut table = Vec::new();
        let table_len = 256;
        let mut time = Time {
            sample_rate: (table_len as f64 / 2.0) as u32,
            tick: 0,
            bpm: time.bpm,
            measure: time.measure.clone(),
            pos: Pos { bar: 0, beat: 0, pos: 0.0 },
        };
        for _i in 0..table_len {
            let v = osc.0.lock().unwrap().proc(&time).0;
            table.push(v);
            time.inc();
        }
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(WaveTable {
                table: Table::new(table),
                ph: ph,
            })
        )))
    }

    pub fn from_table(table: AUnit, ph: AUnit) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Osc(
            Mut::amut(WaveTable {
                table: table,
                ph: ph,
            })
        )))
    }
}

fn linear_interpol(v1: f64, v2: f64, r: f64) -> f64 {
    let r = r % 1.0;
    v1 * r + v2 * (1.0 - r)
}

impl Walk for WaveTable {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        if f(&self.table) { self.table.0.lock().unwrap().walk(f); }
        if f(&self.ph) { self.ph.0.lock().unwrap().walk(f); }
    }
}

impl Unit for WaveTable {
    fn proc(&mut self, time: &Time) -> Signal {
        if let Node::Tab(t) = &self.table.0.lock().unwrap().node {
            let table = &t.0.lock().unwrap().0;
            let len = table.len() as f64;
            let p = self.ph.0.lock().unwrap().proc(&time).0 * len;
            let pos1 = (p.floor() % len) as usize;
            let pos2 = (p.ceil() % len) as usize;
            let v = linear_interpol(table[pos1], table[pos2], p.fract());
            (v, v)
        } else {
            panic!("it's not a table!!")
        }
    }

    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> Dump {
        let mut vec = Vec::new();
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.table)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.table.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        match shared_vec.iter().position(|e| Arc::ptr_eq(e, &self.ph)) {
            Some(idx) => vec.push(Box::new(Dump::Str(shared_map.get(&idx).unwrap().to_string()))),
            None => vec.push(Box::new(self.ph.0.lock().unwrap().dump(shared_vec, shared_map))),
        }
        Dump::Op("wavetable".to_string(), vec)
    }
}

impl Osc for WaveTable {
    fn set_freq(&mut self, freq: AUnit) {
        if let Node::Osc(osc) = &self.ph.0.lock().unwrap().node {
            osc.0.lock().unwrap().set_freq(freq);
        }
    }
}
