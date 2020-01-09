use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use super::super::event::{to_len, to_str, Message};
use super::super::mtime::{Measure, Time};

//// types and traits

pub trait Walk {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool);
}

type OpName = String;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Table(Vec<f64>),
    Pattern(Vec<String>),
    Ug(Aug),
    Shared(usize, Aug),
}

pub struct Slot {
    pub name: String,
    pub value: Value,
    pub ug: Aug,
}

pub enum UgNode {
    Val(Value),
    Ug(OpName, Vec<Slot>),
    UgRest(OpName, Vec<Slot>, String, Vec<Box<Value>>),
}

pub trait Dump: Walk {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode;
}

pub enum OperateError {
    NotUgen,
    CannotParseNumber(String, String),
    ParamNotFound(String),
}

pub trait Operate: Dump {
    fn get(&self, pname: &str) -> Option<Aug>;
    fn get_str(&self, pname: &str) -> Option<String>;
    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError>;
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError>;
    fn clear(&mut self, pname: &str);
}

pub type Signal = (f64, f64);

pub trait Proc: Operate {
    fn proc(&mut self, time: &Time) -> Signal;
}

pub trait Osc: Proc {
    fn get_freq(&self) -> Aug;
    fn set_freq(&mut self, freq: Aug);
}

#[derive(Clone)]
pub enum ADSR {
    Attack,
    Decay,
    Sustin,
    Release,
    None,
}

pub trait Eg: Proc {
    fn set_state(&mut self, state: ADSR, eplaced: u64);
}

pub struct Table(pub Arc<Mutex<Vec<f64>>>);
pub struct Pattern(pub Arc<Mutex<Vec<Box<Message>>>>);

pub enum UG {
    Val(f64),
    Proc(Box<dyn Proc + Send>),
    Osc(Box<dyn Osc + Send>),
    Eg(Box<dyn Eg + Send>),
    Tab(Table),
    Pat(Pattern),
}

pub struct UGen {
    pub id: usize,
    pub last_tick: u64,
    pub last_sig: Signal,
    pub ug: UG,
}

pub struct Aug(pub Arc<Mutex<UGen>>);

// trait implementations for Table

impl Table {
    pub fn new(data: Vec<f64>) -> Table {
        Table(Arc::new(Mutex::new(data)))
    }
}

impl Walk for Table {
    fn walk(&self, _f: &mut dyn FnMut(&Aug) -> bool) {}
}

impl Dump for Table {
    fn dump(&self, _shared_vec: &Vec<Aug>) -> UgNode {
        let mut vec = Vec::new();
        for v in self.0.lock().unwrap().iter() {
            vec.push(*v);
        }
        UgNode::Val(Value::Table(vec))
    }
}

// trait implementations for Pattern

impl Pattern {
    pub fn new(data: Vec<Box<Message>>) -> Pattern {
        Pattern(Arc::new(Mutex::new(data)))
    }
}

impl Walk for Pattern {
    fn walk(&self, _f: &mut dyn FnMut(&Aug) -> bool) {}
}

impl Dump for Pattern {
    fn dump(&self, _shared_vec: &Vec<Aug>) -> UgNode {
        let mut vec = Vec::new();
        let m = Measure { beat: 4, note: 4 };

        for ev in self.0.lock().unwrap().iter() {
            match &**ev {
                Message::Note(pitch, len) => {
                    let pitch_s = to_str(&pitch);
                    let len_s = to_len(&len, &m);
                    vec.push(format!("({} {})", pitch_s, len_s));
                }
                Message::Loop => vec.push("loop".to_string()),
            }
        }
        UgNode::Val(Value::Pattern(vec))
    }
}

// trait implementations for UG

impl Walk for UG {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        match self {
            UG::Val(_) => (),
            UG::Proc(u) => u.walk(f),
            UG::Osc(u) => u.walk(f),
            UG::Eg(u) => u.walk(f),
            UG::Tab(_) => (),
            UG::Pat(_) => (),
        }
    }
}

impl Dump for UG {
    fn dump(&self, shared_vec: &Vec<Aug>) -> UgNode {
        match self {
            UG::Val(v) => UgNode::Val(Value::Number(*v)),
            UG::Proc(u) => u.dump(shared_vec),
            UG::Osc(u) => u.dump(shared_vec),
            UG::Eg(u) => u.dump(shared_vec),
            UG::Tab(t) => t.dump(shared_vec),
            UG::Pat(p) => p.dump(shared_vec),
        }
    }
}

impl Operate for UG {
    fn get(&self, pname: &str) -> Option<Aug> {
        None
    }
    fn get_str(&self, pname: &str) -> Option<String> {
        None
    }
    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for UG {
    fn proc(&mut self, time: &Time) -> Signal {
        match self {
            UG::Val(v) => (*v, *v),
            UG::Proc(u) => u.proc(time),
            UG::Osc(u) => u.proc(time),
            UG::Eg(u) => u.proc(time),
            UG::Tab(_) => (0.0, 0.0),
            UG::Pat(_) => (0.0, 0.0),
        }
    }
}

impl Osc for UG {
    fn set_freq(&mut self, freq: Aug) {
        match self {
            UG::Osc(u) => u.set_freq(freq),
            _ => (),
        }
    }

    fn get_freq(&self) -> Aug {
        match self {
            UG::Osc(u) => u.get_freq(),
            _ => Aug::val(0.0),
        }
    }
}

impl Eg for UG {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        match self {
            UG::Eg(u) => u.set_state(state, eplaced),
            _ => (),
        }
    }
}

// trait implementations for UGen

impl UGen {
    pub fn new(ug: UG) -> UGen {
        UGen {
            id: 0, // FIXME
            last_tick: 0,
            last_sig: (0.0, 0.0),
            ug: ug,
        }
    }
}

impl Walk for UGen {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        self.ug.walk(f);
    }
}

impl Dump for UGen {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        self.ug.dump(shared_ug)
    }
}

impl Operate for UGen {
    fn get(&self, pname: &str) -> Option<Aug> {
        match &self.ug {
            UG::Proc(u) => u.get(pname),
            UG::Proc(u) => u.get(pname),
            UG::Proc(u) => u.get(pname),
            _ => None,
        }
    }

    fn get_str(&self, pname: &str) -> Option<String> {
        match &self.ug {
            UG::Proc(u) => u.get_str(pname),
            UG::Proc(u) => u.get_str(pname),
            UG::Proc(u) => u.get_str(pname),
            _ => None,
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match &mut self.ug {
            UG::Proc(u) => u.set(pname, ug),
            UG::Osc(u) => u.set(pname, ug),
            UG::Eg(u) => u.set(pname, ug),
            _ => Err(OperateError::NotUgen),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match &mut self.ug {
            UG::Proc(u) => u.set_str(pname, data),
            UG::Osc(u) => u.set_str(pname, data),
            UG::Eg(u) => u.set_str(pname, data),
            _ => Err(OperateError::NotUgen),
        }
    }

    fn clear(&mut self, pname: &str) {}
}

impl Proc for UGen {
    fn proc(&mut self, time: &Time) -> Signal {
        if self.last_tick == time.tick {
            self.last_sig
        } else {
            self.last_tick = time.tick;
            let sig = self.ug.proc(time);
            self.last_sig = sig;
            sig
        }
    }
}

// trait implementations for Aug

impl Aug {
    pub fn new(ug: UGen) -> Aug {
        Aug(Arc::new(Mutex::new(ug)))
    }

    pub fn val(v: f64) -> Aug {
        Aug::new(UGen::new(UG::Val(v)))
    }
}

impl Clone for Aug {
    fn clone(&self) -> Aug {
        Aug(self.0.clone())
    }
}

impl PartialEq for Aug {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Aug {}

impl Hash for Aug {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::into_raw(self.0.clone()).hash(state);
    }
}

impl Walk for Aug {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        (*self.0.lock().unwrap()).walk(f)
    }
}

impl Dump for Aug {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        self.0.lock().unwrap().dump(shared_ug)
    }
}

impl Operate for Aug {
    fn get(&self, pname: &str) -> Option<Aug> {
        self.0.lock().unwrap().get(pname)
    }
    fn get_str(&self, pname: &str) -> Option<String> {
        self.0.lock().unwrap().get_str(pname)
    }
    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        self.0.lock().unwrap().set(pname, ug)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        self.0.lock().unwrap().set_str(pname, data)
    }
    fn clear(&mut self, pname: &str) {}
}

impl Proc for Aug {
    fn proc(&mut self, time: &Time) -> Signal {
        self.0.lock().unwrap().proc(time)
    }
}
