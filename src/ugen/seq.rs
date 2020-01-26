use std::collections::VecDeque;

use super::super::event::{to_freq, Event, Message, Pitch};
use super::super::mtime::{Measure, Pos, PosOps, Time};

use super::core::{
    Aug, Dump, Eg, Operate, OperateError, Pattern, Proc, Signal, Slot, UGen, UgNode, Value, Walk,
    ADSR, UG,
};
use super::misc::Add;

pub struct Trigger {
    eg: Aug,
    egs: Vec<Aug>,
}

impl Trigger {
    pub fn new(eg: Aug, egs: Vec<Aug>) -> Aug {
        Aug::new(UGen::new(UG::Eg(Box::new(Trigger { eg: eg, egs: egs }))))
    }
}

impl Walk for Trigger {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.eg) {
            self.eg.walk(f);
        }
        for eg in &self.egs {
            if f(eg) {
                eg.walk(f);
            }
        }
    }
}

impl Dump for Trigger {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();
        let mut values = Vec::new();

        slots.push(Slot {
            ug: self.eg.clone(),
            name: "eg".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.eg) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.eg.clone()),
            },
        });

        for eg in self.egs.iter() {
            values.push(match shared_ug.iter().position(|e| *e == *eg) {
                Some(n) => Box::new(Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone())),
                None => Box::new(Value::Ug(eg.clone())),
            });
        }

        UgNode::UgRest("trig".to_string(), slots, "src".to_string(), values)
    }
}

impl Operate for Trigger {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "eg" => Ok(self.eg.clone()),
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    Ok(self.egs[idx].clone())
                } else {
                    Err(OperateError::ParamNotFound(format!("trig/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("trig/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "trig/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "eg" => {
                self.eg = ug;
                Ok(true)
            }
            name if name.starts_with("src") => {
                if let Ok(idx) = name[3..].to_string().parse::<usize>() {
                    while self.egs.len() <= idx {
                        self.egs.push(Aug::val(0.0));
                    }
                    self.egs[idx] = ug;
                    Ok(true)
                } else {
                    Err(OperateError::ParamNotFound(format!("trig/{}", pname)))
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("trig/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        let mut data = data.clone();
        data.retain(|c| c != '\n' && c != ' ');

        match pname {
            "eg" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.eg = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("trig/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("trig/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "eg" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            name if name.starts_with("src") => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Trigger {
    fn proc(&mut self, time: &Time) -> Signal {
        for eg in &mut self.egs {
            eg.proc(&time);
        }
        self.eg.proc(&time)
    }
}

impl Eg for Trigger {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
            eg.set_state(state.clone(), eplaced);
        }
        for eg in &self.egs {
            if let UG::Eg(ref mut eg) = &mut eg.0.lock().unwrap().ug {
                eg.set_state(state.clone(), eplaced);
            }
        }
    }
}

pub struct AdsrEg {
    a: Aug,
    d: Aug,
    s: Aug,
    r: Aug,
    state: ADSR,
    eplaced: u64,
}

impl AdsrEg {
    pub fn new(a: Aug, d: Aug, s: Aug, r: Aug) -> Aug {
        Aug::new(UGen::new(UG::Eg(Box::new(AdsrEg {
            a: a,
            d: d,
            s: s,
            r: r,
            state: ADSR::None,
            eplaced: 0,
        }))))
    }
}

fn sec_to_sample_num(sec: f64, time: &Time) -> u64 {
    (time.sample_rate as f64 * sec) as u64
}

impl Walk for AdsrEg {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.a) {
            self.a.walk(f);
        }
        if f(&self.d) {
            self.d.walk(f);
        }
        if f(&self.s) {
            self.s.walk(f);
        }
        if f(&self.r) {
            self.r.walk(f);
        }
    }
}

impl Dump for AdsrEg {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.a.clone(),
            name: "a".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.a) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.a.clone()),
            },
        });
        slots.push(Slot {
            ug: self.d.clone(),
            name: "d".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.d) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.d.clone()),
            },
        });
        slots.push(Slot {
            ug: self.s.clone(),
            name: "s".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.s) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.s.clone()),
            },
        });
        slots.push(Slot {
            ug: self.r.clone(),
            name: "r".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.r) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.r.clone()),
            },
        });

        UgNode::Ug("adsr".to_string(), slots)
    }
}

impl Operate for AdsrEg {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "a" => Ok(self.a.clone()),
            "d" => Ok(self.d.clone()),
            "s" => Ok(self.s.clone()),
            "r" => Ok(self.r.clone()),
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
                        "adsr/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "a" => {
                self.a = ug;
                Ok(true)
            }
            "d" => {
                self.d = ug;
                Ok(true)
            }
            "s" => {
                self.s = ug;
                Ok(true)
            }
            "r" => {
                self.r = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("adsr/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        let mut data = data.clone();
        data.retain(|c| c != '\n' && c != ' ');

        match pname {
            "a" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.a = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("adsr/{}", pname), data.clone());
                    Err(err)
                }
            }
            "d" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.d = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("adsr/{}", pname), data.clone());
                    Err(err)
                }
            }
            "s" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.s = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("adsr/{}", pname), data.clone());
                    Err(err)
                }
            }
            "r" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.r = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("adsr/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("adsr/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "a" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "d" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "s" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "r" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for AdsrEg {
    fn proc(&mut self, time: &Time) -> Signal {
        let a = sec_to_sample_num(self.a.proc(time).0, time);
        let d = sec_to_sample_num(self.d.proc(time).0, time);
        let s = self.s.proc(time).0;
        let r = sec_to_sample_num(self.r.proc(time).0, time);
        let state = &self.state;
        let eplaced = self.eplaced;
        let v;

        match state {
            ADSR::Attack => {
                if eplaced < a {
                    v = self.eplaced as f64 / a as f64;
                } else if eplaced < a + d {
                    v = 1.0 - (1.0 - s) * ((eplaced as f64 - a as f64) / d as f64);
                    self.state = ADSR::Decay;
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            }
            ADSR::Decay => {
                if eplaced < a + d {
                    v = 1.0 - (1.0 - s) * ((eplaced as f64 - a as f64) / d as f64);
                } else if eplaced >= a + d {
                    v = s;
                    self.state = ADSR::Sustin;
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            }
            ADSR::Sustin => {
                v = s;
            }
            ADSR::Release => {
                if eplaced < r {
                    v = s - eplaced as f64 * (s / r as f64);
                } else {
                    v = 0.0;
                    self.state = ADSR::None;
                }
            }
            ADSR::None => {
                v = 0.0;
            }
        }
        self.eplaced += 1;
        (v, v)
    }
}

impl Eg for AdsrEg {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        self.state = state;
        self.eplaced = eplaced;
    }
}

pub struct Seq {
    pattern: Aug,
    queue: VecDeque<Box<Event>>,
    osc: Aug,
    osc_mod: Aug,
    eg: Aug,
}

impl Seq {
    pub fn new(pat: Aug, osc: Aug, osc_mod: Aug, eg: Aug, time: &Time) -> Aug {
        let mut seq = Seq {
            pattern: pat,
            queue: VecDeque::new(),
            osc: osc,
            osc_mod: osc_mod,
            eg: eg,
        };
        seq.fill_queue(&time.pos, &time.measure);
        Aug::new(UGen::new(UG::Proc(Box::new(seq))))
    }

    pub fn fill_queue(&mut self, base: &Pos, measure: &Measure) {
        let mut pos = base.clone();
        if let UG::Pat(pat) = &self.pattern.0.lock().unwrap().ug {
            for m in pat.0.lock().unwrap().iter() {
                match &**m {
                    Message::Note(pitch, len) => match pitch {
                        Pitch::Pitch(_, _) => {
                            self.queue
                                .push_back(Box::new(Event::On(pos.clone(), to_freq(pitch))));
                            pos = pos.clone().add(len.clone(), &measure);
                            self.queue.push_back(Box::new(Event::Off(pos.clone())));
                        }
                        Pitch::Kick => {
                            self.queue.push_back(Box::new(Event::Kick(pos.clone())));
                            pos = pos.clone().add(len.clone(), &measure);
                            self.queue.push_back(Box::new(Event::Off(pos.clone())));
                        }
                        Pitch::Rest => {
                            pos = pos.clone().add(len.clone(), &measure);
                        }
                    },
                    Message::Loop => {
                        self.queue.push_back(Box::new(Event::Loop(pos.clone())));
                    }
                }
            }
        } else {
            println!("aug is not a pattern!!");
        }
    }
}

impl Walk for Seq {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.pattern) {
            self.pattern.walk(f);
        }
        if f(&self.osc) {
            self.osc.walk(f);
        }
        if f(&self.osc_mod) {
            self.osc_mod.walk(f);
        }
        if f(&self.eg) {
            self.eg.walk(f);
        }
    }
}

impl Dump for Seq {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.pattern.clone(),
            name: "pattern".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.pattern) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.pattern.clone()),
            },
        });
        slots.push(Slot {
            ug: self.osc.clone(),
            name: "osc".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.osc) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.osc.clone()),
            },
        });
        slots.push(Slot {
            ug: self.osc_mod.clone(),
            name: "osc_mod".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.osc_mod) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.osc_mod.clone()),
            },
        });
        slots.push(Slot {
            ug: self.eg.clone(),
            name: "eg".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.eg) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.eg.clone()),
            },
        });

        UgNode::Ug("seq".to_string(), slots)
    }
}

impl Operate for Seq {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "pattern" => Ok(self.pattern.clone()),
            "osc" => Ok(self.osc.clone()),
            "osc_mod" => Ok(self.osc_mod.clone()),
            "eg" => Ok(self.eg.clone()),
            _ => Err(OperateError::ParamNotFound(format!("seq/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "seq/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "pattern" => {
                self.pattern = ug;
                Ok(true)
            }
            "osc" => {
                self.osc = ug;
                Ok(true)
            }
            "osc_mod" => {
                self.osc_mod = ug;
                Ok(true)
            }
            "eg" => {
                self.eg = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("seq/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match pname {
            "pattern" => {
                let mut data = data.clone();
                data.retain(|c| c != '\n');

                if let Ok(msgs) = Pattern::parse_str(data.clone()) {
                    self.pattern = Aug::new(UGen::new(UG::Pat(Pattern::new(msgs))));
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParsePattern(format!("seq/{}", pname), data.clone());
                    Err(err)
                }
            }
            "osc" => {
                let mut data = data.clone();
                data.retain(|c| c != '\n' && c != ' ');

                if let Ok(v) = data.parse::<f64>() {
                    self.osc = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("seq/{}", pname), data.clone());
                    Err(err)
                }
            }
            "osc_mod" => {
                let mut data = data.clone();
                data.retain(|c| c != '\n' && c != ' ');

                if let Ok(v) = data.parse::<f64>() {
                    self.osc_mod = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("seq/{}", pname), data.clone());
                    Err(err)
                }
            }
            "eg" => {
                let mut data = data.clone();
                data.retain(|c| c != '\n' && c != ' ');

                if let Ok(v) = data.parse::<f64>() {
                    self.eg = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("seq/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("seq/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "pattern" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "osc" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "osc_mod" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "eg" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for Seq {
    fn proc(&mut self, time: &Time) -> Signal {
        self.osc_mod.proc(&time);
        let (ol, or) = self.osc.proc(&time);
        let (el, er) = self.eg.proc(&time);
        let mut q = self.queue.iter().peekable();

        match q.peek() {
            Some(e) => match &***e {
                Event::On(pos, _freq) => {
                    if pos <= &time.pos {
                        if let Event::On(_pos, freq) = *self.queue.pop_front().unwrap() {
                            if let UG::Osc(ref mut osc) = &mut self.osc.0.lock().unwrap().ug {
                                let freq =
                                    vec![self.osc_mod.clone(), Aug::new(UGen::new(UG::Val(freq)))];
                                osc.set_freq(Add::new(freq));
                            }
                            if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
                                eg.set_state(ADSR::Attack, 0);
                            }
                        }
                    }
                }
                Event::Kick(pos) => {
                    if pos <= &time.pos {
                        if let Event::Kick(_pos) = *self.queue.pop_front().unwrap() {
                            if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
                                eg.set_state(ADSR::Attack, 0);
                            }
                        }
                    }
                }
                Event::Off(pos) => {
                    if pos <= &time.pos {
                        if let Event::Off(_pos) = *self.queue.pop_front().unwrap() {
                            if let UG::Eg(ref mut eg) = &mut self.eg.0.lock().unwrap().ug {
                                eg.set_state(ADSR::Release, 0);
                            }
                        }
                    }
                }
                Event::Loop(pos) => {
                    if pos <= &time.pos {
                        let base = Pos {
                            bar: time.pos.bar,
                            beat: 0,
                            pos: 0.0,
                        };
                        self.queue.pop_front().unwrap();
                        self.fill_queue(&base, &time.measure);
                    }
                }
            },
            None => (),
        }
        ((ol * el), (or * er))
    }
}
