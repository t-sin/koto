use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::super::event::{to_len, to_str, Message};
use super::super::mtime::{Measure, Time};

//// types and traits

pub trait Walk {
    fn walk(&self, f: &mut Fn(&Aug) -> bool);
}

type OpName = String;
type ParamName = String;

#[derive(Debug)]
pub enum Parameter {
    Value(f64),
    Table(Vec<f64>),
    Pattern(Vec<String>),
    UG(OpName, Vec<ParamName>, Vec<Box<Parameter>>),
}

pub trait Dump: Walk {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Parameter;
}

pub type Signal = (f64, f64);

pub trait Proc: Dump {
    fn proc(&mut self, time: &Time) -> Signal;
}

pub trait Osc: Proc {
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
    Proc(Proc),
    Osc(Osc),
    Eg(Eg),
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

impl Dump for Table {
    fn dump(&self, _shared_vec: &Vec<Aug>, _shared_map: &HashMap<usize, String>) -> Parameter {
        let mut vec = Vec::new();
        for v in self.0.iter() {
            vec.push(v.to_string());
        }
        Parameter::Table(vec)
    }
}

// trait implementations for Pattern

impl Pattern {
    pub fn new(data: Vec<Box<Message>>) -> Pattern {
        Pattern(Arc::new(Mutex::new(data)))
    }
}

impl Dump for Pattern {
    fn dump(&self, _shared_vec: &Vec<Aug>, _shared_map: &HashMap<usize, String>) -> Parameter {
        let mut vec = Vec::new();
        let m = Measure { beat: 4, note: 4 };

        for ev in self.0.iter() {
            match &**ev {
                Message::Note(pitch, len) => {
                    let pitch_s = to_str(&pitch);
                    let len_s = to_len(&len, &m);
                    vec.push(format!("({} {})", pitch_s, len_s));
                }
                Message::Loop => vec.push("loop".to_string()),
            }
        }
        Parameter::Pattern(vec)
    }
}

// trait implementations for UG

impl Walk for UG {
    fn walk(&self, f: &mut Fn(&Aug) -> bool) {
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
    fn dump(&self, shared_vec: &Vec<Aug>, shared_map: &HashMap<usize, String>) -> Parameter {
        match self {
            UG::Val(v) => Parameter::Value(v.to_string()),
            UG::Sig(u) => u.dump(shared_vec, shared_map),
            UG::Osc(u) => u.dump(shared_vec, shared_map),
            UG::Eg(u) => u.dump(shared_vec, shared_map),
            UG::Tab(t) => t.dump(shared_vec, shared_map),
            UG::Pat(p) => p.dump(shared_vec, shared_map),
        }
    }
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

static mut id: usize = 0;

impl UGen {
    pub fn new(ug: UG) -> UGen {
        let ug = UGen {
            id: id,
            last_tick: 0,
            last_sig: (0.0, 0.0),
            ug: ug,
        };
        id += 1;
        ug
    }
}

impl PartialEq for UGen {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for UGen {}

// trait implementations for Aug

impl Aug {
    pub fn new(ug: UGen) -> Aug {
        Aug(Arc::new(Mutex::new(ug)))
    }
}

impl PartialEq for Aug {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self.0, other.0)
    }
}

impl Eq for Aug {}

impl Walk for Aug {
    fn walk(&self, f: &mut Fn(&Aug)) -> bool {
        self.0.lock().unwrap().walk(f)
    }
}

impl Dump for Aug {
    fn dump(&self, shared_ug: &Vec<Aug>) -> Parameter {
        self.0.lock().unwrap().dump(shared_ug)
    }
}
