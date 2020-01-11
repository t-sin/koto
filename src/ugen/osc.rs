use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::super::mtime::{Clock, Pos, Time};
use super::core::{
    Aug, Dump, Operate, OperateError, Osc, Proc, Signal, Slot, Table, UGen, UgNode, Value, Walk, UG,
};
use super::misc::{Clip, Gain, Offset};

pub struct Rand {
    rng: SmallRng,
    v: f64,
}

impl Rand {
    pub fn new(seed: u64) -> Aug {
        Aug::new(UGen::new(UG::Osc(Box::new(Rand {
            rng: SmallRng::seed_from_u64(seed),
            v: 0.15,
        }))))
    }
}

impl Walk for Rand {
    fn walk(&self, _f: &mut dyn FnMut(&Aug) -> bool) {}
}

impl Dump for Rand {
    fn dump(&self, _shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        UgNode::Ug("rand".to_string(), slots)
    }
}

impl Operate for Rand {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "rand/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Rand {
    fn proc(&mut self, _time: &Time) -> Signal {
        self.v = self.rng.gen();
        (self.v, self.v)
    }
}

impl Osc for Rand {
    fn set_freq(&mut self, _u: Aug) {}
    fn get_freq(&self) -> Aug {
        Aug::val(0.0)
    }
}

pub struct Sine {
    pub init_ph: Aug,
    pub ph: f64,
    pub freq: Aug,
}

impl Sine {
    pub fn new(init_ph: Aug, freq: Aug) -> Aug {
        Aug::new(UGen::new(UG::Osc(Box::new(Sine {
            init_ph: init_ph,
            ph: 0.0,
            freq: freq,
        }))))
    }
}

impl Walk for Sine {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.walk(f);
        }
        if f(&self.freq) {
            self.freq.walk(f);
        }
    }
}

impl Dump for Sine {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.init_ph.clone(),
            name: "init_ph".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.init_ph) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.init_ph.clone()),
            },
        });
        slots.push(Slot {
            ug: self.freq.clone(),
            name: "freq".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.freq) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.freq.clone()),
            },
        });

        UgNode::Ug("sine".to_string(), slots)
    }
}

impl Operate for Sine {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "sine/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Sine {
    fn proc(&mut self, time: &Time) -> Signal {
        let init_ph = self.init_ph.proc(&time).0;
        let v = (init_ph + self.ph).sin();
        let ph_diff = time.sample_rate as f64 / std::f64::consts::PI;
        self.ph += self.freq.proc(&time).0 / ph_diff;

        (v, v)
    }
}

impl Osc for Sine {
    fn set_freq(&mut self, u: Aug) {
        self.freq = u;
    }

    fn get_freq(&self) -> Aug {
        self.freq.clone()
    }
}

pub struct Tri {
    pub init_ph: Aug,
    pub ph: f64,
    pub freq: Aug,
}

impl Tri {
    pub fn new(init_ph: Aug, freq: Aug) -> Aug {
        Aug::new(UGen::new(UG::Osc(Box::new(Tri {
            init_ph: init_ph,
            ph: 0.0,
            freq: freq,
        }))))
    }
}

impl Walk for Tri {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.walk(f);
        }
        if f(&self.freq) {
            self.freq.walk(f);
        }
    }
}

impl Dump for Tri {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.init_ph.clone(),
            name: "init_ph".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.init_ph) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.init_ph.clone()),
            },
        });
        slots.push(Slot {
            ug: self.freq.clone(),
            name: "freq".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.freq) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.freq.clone()),
            },
        });

        UgNode::Ug("tri".to_string(), slots)
    }
}

impl Operate for Tri {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "tri/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Tri {
    fn proc(&mut self, time: &Time) -> Signal {
        let ph = self.init_ph.proc(&time).0 + self.ph;

        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.proc(&time).0 / ph_diff;

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

impl Osc for Tri {
    fn set_freq(&mut self, u: Aug) {
        self.freq = u;
    }

    fn get_freq(&self) -> Aug {
        self.freq.clone()
    }
}

pub struct Saw {
    pub init_ph: Aug,
    pub ph: f64,
    pub freq: Aug,
}

impl Saw {
    pub fn new(init_ph: Aug, freq: Aug) -> Aug {
        Aug::new(UGen::new(UG::Osc(Box::new(Saw {
            init_ph: init_ph,
            ph: 0.0,
            freq: freq,
        }))))
    }
}

impl Walk for Saw {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.walk(f);
        }
        if f(&self.freq) {
            self.freq.walk(f);
        }
    }
}

impl Dump for Saw {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.init_ph.clone(),
            name: "init_ph".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.init_ph) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.init_ph.clone()),
            },
        });
        slots.push(Slot {
            ug: self.freq.clone(),
            name: "freq".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.freq) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.freq.clone()),
            },
        });

        UgNode::Ug("saw".to_string(), slots)
    }
}

impl Operate for Saw {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "saw/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Saw {
    fn proc(&mut self, time: &Time) -> Signal {
        let ph = self.init_ph.proc(&time).0 + self.ph;
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.proc(&time).0 / ph_diff;

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

impl Osc for Saw {
    fn set_freq(&mut self, u: Aug) {
        self.freq = u;
    }

    fn get_freq(&self) -> Aug {
        self.freq.clone()
    }
}

pub struct Pulse {
    pub init_ph: Aug,
    pub ph: f64,
    pub freq: Aug,
    pub duty: Aug,
}

impl Pulse {
    pub fn new(init_ph: Aug, freq: Aug, duty: Aug) -> Aug {
        Aug::new(UGen::new(UG::Osc(Box::new(Pulse {
            init_ph: init_ph,
            ph: 0.0,
            freq: freq,
            duty: duty,
        }))))
    }
}

impl Walk for Pulse {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.init_ph) {
            self.init_ph.walk(f);
        }
        if f(&self.freq) {
            self.freq.walk(f);
        }
        if f(&self.duty) {
            self.duty.walk(f);
        }
    }
}

impl Dump for Pulse {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.init_ph.clone(),
            name: "init_ph".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.init_ph) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.init_ph.clone()),
            },
        });
        slots.push(Slot {
            ug: self.freq.clone(),
            name: "freq".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.freq) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.freq.clone()),
            },
        });
        slots.push(Slot {
            ug: self.duty.clone(),
            name: "duty".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.duty) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.duty.clone()),
            },
        });

        UgNode::Ug("pulse".to_string(), slots)
    }
}

impl Operate for Pulse {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "pulse/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Pulse {
    fn proc(&mut self, time: &Time) -> Signal {
        let ph = self.init_ph.proc(&time).0 + self.ph;
        let duty = self.duty.proc(&time).0;
        let ph_diff = time.sample_rate as f64 * 2.0;
        self.ph += self.freq.proc(&time).0 / ph_diff;

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

impl Osc for Pulse {
    fn set_freq(&mut self, u: Aug) {
        self.freq = u;
    }

    fn get_freq(&self) -> Aug {
        self.freq.clone()
    }
}

pub struct Phase {
    pub root: Aug,
    pub osc: Aug,
}

impl Phase {
    pub fn new(u: Aug) -> Aug {
        let offset_val = Aug::val(1.0);
        let gain = Aug::val(0.5);
        let clip_min = Aug::val(-1.0);
        let clip_max = Aug::val(1.0);
        Aug::new(UGen::new(UG::Osc(Box::new(Phase {
            root: Offset::new(
                offset_val,
                Gain::new(gain, Clip::new(clip_min, clip_max, u.clone())),
            ),
            osc: u.clone(),
        }))))
    }
}

impl Walk for Phase {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.osc) {
            self.osc.walk(f);
        }
    }
}

impl Dump for Phase {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.osc.clone(),
            name: "osc".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.osc) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.osc.clone()),
            },
        });

        UgNode::Ug("phase".to_string(), slots)
    }
}

impl Operate for Phase {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "phase/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Phase {
    fn proc(&mut self, time: &Time) -> Signal {
        self.root.proc(time)
    }
}

impl Osc for Phase {
    fn set_freq(&mut self, freq: Aug) {
        if let UG::Osc(ref mut osc) = &mut self.osc.0.lock().unwrap().ug {
            osc.set_freq(freq);
        }
    }

    fn get_freq(&self) -> Aug {
        Aug::val(0.0)
    }
}

pub struct WaveTable {
    pub table: Aug,
    pub ph: Aug,
}

impl WaveTable {
    pub fn from_osc(osc: Aug, ph: Aug, time: &Time) -> Aug {
        let mut table = Vec::new();
        let table_len = 256;
        let mut time = Time {
            sample_rate: (table_len as f64 / 2.0) as u32,
            tick: 0,
            bpm: time.bpm,
            measure: time.measure.clone(),
            pos: Pos {
                bar: 0,
                beat: 0,
                pos: 0.0,
            },
        };
        for _i in 0..table_len {
            let v = osc.0.lock().unwrap().proc(&time).0;
            table.push(v);
            time.inc();
        }
        let table = Aug::new(UGen::new(UG::Tab(Table::new(table))));
        Aug::new(UGen::new(UG::Osc(Box::new(WaveTable {
            table: table,
            ph: ph,
        }))))
    }

    pub fn from_table(table: Aug, ph: Aug) -> Aug {
        Aug::new(UGen::new(UG::Osc(Box::new(WaveTable {
            table: table,
            ph: ph,
        }))))
    }
}

fn linear_interpol(v1: f64, v2: f64, r: f64) -> f64 {
    let r = r % 1.0;
    v1 * r + v2 * (1.0 - r)
}

impl Walk for WaveTable {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.table) {
            self.table.walk(f);
        }
        if f(&self.ph) {
            self.ph.walk(f);
        }
    }
}

impl Dump for WaveTable {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.table.clone(),
            name: "table".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.table) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.table.clone()),
            },
        });

        slots.push(Slot {
            ug: self.ph.clone(),
            name: "ph".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.ph) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.ph.clone()),
            },
        });

        UgNode::Ug("wavetable".to_string(), slots)
    }
}

impl Operate for WaveTable {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        None
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "wavetable/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for WaveTable {
    fn proc(&mut self, time: &Time) -> Signal {
        if let UG::Tab(table) = &self.table.0.lock().unwrap().ug {
            let table = table.0.lock().unwrap();
            let len = table.len() as f64;
            let p = self.ph.proc(&time).0 * len;
            let pos1 = (p.floor() % len) as usize;
            let pos2 = (p.ceil() % len) as usize;
            let v = linear_interpol(table[pos1], table[pos2], p.fract());
            (v, v)
        } else {
            panic!("it's not a table!!");
        }
    }
}

impl Osc for WaveTable {
    fn set_freq(&mut self, freq: Aug) {
        if let UG::Osc(ref mut osc) = &mut self.ph.0.lock().unwrap().ug {
            osc.set_freq(freq);
        }
    }

    fn get_freq(&self) -> Aug {
        Aug::val(0.0)
    }
}
