extern crate num;

use super::super::mtime::Time;
use super::core::{
    Aug, Dump, Operate, OperateError, Proc, Signal, Slot, UGen, UgNode, Value, Walk, UG,
};

pub struct Pan {
    pub pan: Aug,
    pub src: Aug,
}

impl Pan {
    pub fn new(pan: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Pan { pan: pan, src: src }))))
    }
}

impl Walk for Pan {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.pan) {
            self.pan.walk(f);
        }
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for Pan {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.pan.clone(),
            name: "pan".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.pan) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.pan.clone()),
            },
        });
        slots.push(Slot {
            ug: self.src.clone(),
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("pan".to_string(), slots)
    }
}

impl Operate for Pan {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "pan" => Ok(self.pan.clone()),
            "src" => Ok(self.src.clone()),
            _ => Err(OperateError::ParamNotFound(format!("pan/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "pan/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "pan" => {
                self.pan = ug;
                Ok(true)
            }
            "src" => {
                self.src = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("pan/{}", pname))),
        }
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            "pan" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.pan = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("delay/{}", pname), data.clone());
                    Err(err)
                }
            }
            "src" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.src = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("delay/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("delay/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "pan" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "src" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Pan {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        let v = self.pan.proc(&time).0;

        if v > 0.0 {
            (l * (1.0 - v), r)
        } else if v < 0.0 {
            (l, r * (1.0 - v))
        } else {
            (l, r)
        }
    }
}

pub struct Clip {
    pub min: Aug,
    pub max: Aug,
    pub src: Aug,
}

impl Clip {
    pub fn new(min: Aug, max: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Clip {
            min: min,
            max: max,
            src: src,
        }))))
    }
}

impl Walk for Clip {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.min) {
            self.min.walk(f);
        }
        if f(&self.max) {
            self.max.walk(f);
        }
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for Clip {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.min.clone(),
            name: "min".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.min) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.min.clone()),
            },
        });
        slots.push(Slot {
            ug: self.max.clone(),
            name: "max".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.max) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.max.clone()),
            },
        });
        slots.push(Slot {
            ug: self.src.clone(),
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("clip".to_string(), slots)
    }
}

impl Operate for Clip {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "min" => Ok(self.min.clone()),
            "max" => Ok(self.max.clone()),
            "src" => Ok(self.src.clone()),
            _ => Err(OperateError::ParamNotFound(format!("clip/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "clip/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "min" => {
                self.min = ug;
                Ok(true)
            }
            "max" => {
                self.max = ug;
                Ok(true)
            }
            "src" => {
                self.src = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("clip/{}", pname))),
        }
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            "min" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.min = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("clip/{}", pname), data.clone());
                    Err(err)
                }
            }
            "max" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.max = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("clip/{}", pname), data.clone());
                    Err(err)
                }
            }
            "src" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.src = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("clip/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("clip/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "min" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "max" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "src" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Clip {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        let min = self.min.proc(&time).0;
        let max = self.max.proc(&time).0;
        (num::clamp(l, min, max), num::clamp(r, min, max))
    }
}

pub struct Offset {
    pub val: Aug,
    pub src: Aug,
}

impl Offset {
    pub fn new(val: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Offset { val: val, src: src }))))
    }
}

impl Walk for Offset {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.val) {
            self.val.walk(f);
        }
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for Offset {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.val.clone(),
            name: "val".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.val) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.val.clone()),
            },
        });
        slots.push(Slot {
            ug: self.src.clone(),
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("offset".to_string(), slots)
    }
}

impl Operate for Offset {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "val" => Ok(self.val.clone()),
            "src" => Ok(self.src.clone()),
            _ => Err(OperateError::ParamNotFound(format!("offset/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "offset/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "val" => {
                self.val = ug;
                Ok(true)
            }
            "src" => {
                self.src = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("offset/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            "val" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.val = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("offset/{}", pname), data.clone());
                    Err(err)
                }
            }
            "src" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.src = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("offset/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("offset/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "val" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "src" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Offset {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        let val = self.val.proc(&time).0;
        (l + val, r + val)
    }
}

pub struct Gain {
    pub gain: Aug,
    pub src: Aug,
}

impl Gain {
    pub fn new(gain: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Gain {
            gain: gain,
            src: src,
        }))))
    }
}

impl Walk for Gain {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for Gain {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.gain.clone(),
            name: "gain".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.gain) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.gain.clone()),
            },
        });
        slots.push(Slot {
            ug: self.src.clone(),
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("gain".to_string(), slots)
    }
}

impl Operate for Gain {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "gain" => Ok(self.gain.clone()),
            "src" => Ok(self.src.clone()),
            _ => Err(OperateError::ParamNotFound(format!("gain/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "gain/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "gain" => {
                self.gain = ug;
                Ok(true)
            }
            "src" => {
                self.src = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("gain/{}", pname))),
        }
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            "gain" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.gain = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("gain/{}", pname), data.clone());
                    Err(err)
                }
            }
            "src" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.src = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("gain/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("gain/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "gain" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "src" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Gain {
    fn proc(&mut self, time: &Time) -> Signal {
        let (l, r) = self.src.proc(&time);
        let gain = self.gain.proc(&time).0;
        (l * gain, r * gain)
    }
}

pub struct Add {
    pub sources: Vec<Aug>,
}

impl Add {
    pub fn new(sources: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Add { sources: sources }))))
    }
}

impl Walk for Add {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) {
                s.walk(f);
            }
        }
    }
}

impl Dump for Add {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut values = Vec::new();

        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => values.push(Box::new(Value::Shared(
                    n,
                    shared_ug.iter().nth(n).unwrap().clone(),
                ))),
                None => values.push(Box::new(Value::Ug(u.clone()))),
            };
        }
        UgNode::UgRest("+".to_string(), Vec::new(), "src".to_string(), values)
    }
}

impl Operate for Add {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    Ok(self.sources[idx].clone())
                } else {
                    Err(OperateError::ParamNotFound(format!("add/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("add/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "add/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    while self.sources.len() <= idx {
                        self.sources.push(Aug::val(0.0));
                    }
                    self.sources[idx] = ug;
                    Ok(true)
                } else {
                    Err(OperateError::ParamNotFound(format!("add/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("add/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            name if name.starts_with("src") => {
                if let Ok(val) = data.parse::<f64>() {
                    self.set(pname, Aug::val(val))
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("add/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("add/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            name if name.starts_with("src") => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Add {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.proc(&time);
            l += l2;
            r += r2;
        }
        (l, r)
    }
}

pub struct Multiply {
    pub sources: Vec<Aug>,
}

impl Multiply {
    pub fn new(sources: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Multiply { sources: sources }))))
    }
}

impl Walk for Multiply {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        for s in self.sources.iter() {
            if f(s) {
                s.walk(f);
            }
        }
    }
}

impl Dump for Multiply {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut values = Vec::new();

        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => values.push(Box::new(Value::Shared(
                    n,
                    shared_ug.iter().nth(n).unwrap().clone(),
                ))),
                None => values.push(Box::new(Value::Ug(u.clone()))),
            };
        }

        UgNode::UgRest("*".to_string(), Vec::new(), "src".to_string(), values)
    }
}

impl Operate for Multiply {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    Ok(self.sources[idx].clone())
                } else {
                    Err(OperateError::ParamNotFound(format!("mul/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("mul/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "mul/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    while self.sources.len() <= idx {
                        self.sources.push(Aug::val(0.0));
                    }
                    self.sources[idx] = ug;
                    Ok(true)
                } else {
                    Err(OperateError::ParamNotFound(format!("mul/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("mul/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            name if name.starts_with("src") => {
                if let Ok(val) = data.parse::<f64>() {
                    self.set(pname, Aug::val(val))
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("mul/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("mul/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            name if name.starts_with("src") => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Multiply {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 1.0;
        let mut r = 1.0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.proc(&time);
            l *= l2;
            r *= r2;
        }
        (l, r)
    }
}

pub struct Out {
    vol: Aug,
    sources: Vec<Aug>,
}

impl Out {
    pub fn new(vol: Aug, sources: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(Out {
            vol: vol,
            sources: sources,
        }))))
    }
}

impl Walk for Out {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.vol) {
            self.vol.walk(f);
        }
        for s in self.sources.iter() {
            if f(s) {
                s.walk(f);
            }
        }
    }
}

impl Dump for Out {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();
        let mut values = Vec::new();

        slots.push(Slot {
            ug: self.vol.clone(),
            name: "vol".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.vol) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.vol.clone()),
            },
        });

        for u in self.sources.iter() {
            match shared_ug.iter().position(|e| *e == *u) {
                Some(n) => values.push(Box::new(Value::Shared(
                    n,
                    shared_ug.iter().nth(n).unwrap().clone(),
                ))),
                None => values.push(Box::new(Value::Ug(u.clone()))),
            }
        }
        UgNode::UgRest("out".to_string(), slots, "src".to_string(), values)
    }
}

impl Operate for Out {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "vol" => Ok(self.vol.clone()),
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    Ok(self.sources[idx].clone())
                } else {
                    Err(OperateError::ParamNotFound(format!("out/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("out/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "out/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "vol" => {
                self.vol = ug;
                Ok(true)
            }
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    while self.sources.len() <= idx {
                        self.sources.push(Aug::val(0.0));
                    }
                    self.sources[idx] = ug;
                    Ok(true)
                } else {
                    Err(OperateError::ParamNotFound(format!("out/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("out/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        let mut data = data.clone();
        data.retain(|c| c != '\n' && c != ' ');

        match pname {
            "vol" => {
                if let Ok(vol) = data.parse::<f64>() {
                    self.vol = Aug::val(vol);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("out/{}", pname), data.clone());
                    Err(err)
                }
            }
            name if name.starts_with("src") => {
                if let Ok(val) = data.parse::<f64>() {
                    self.set(pname, Aug::val(val))
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("out/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("out/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "vol" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            name if name.starts_with("src") => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Out {
    fn proc(&mut self, time: &Time) -> Signal {
        let mut l = 0.0;
        let mut r = 0.0;
        let vol = self.vol.proc(&time).0;
        for u in self.sources.iter_mut() {
            let (l2, r2) = u.proc(&time);
            l += l2;
            r += r2;
        }
        (l * vol, r * vol)
    }
}
