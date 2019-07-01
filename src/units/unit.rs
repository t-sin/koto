use std::collections::HashMap;
use std::cmp::{PartialEq, Eq};
use std::sync::{Arc, Mutex};

use super::super::time::Time;
use super::super::event::{Message, to_str, to_len};

pub struct Mut<T: ?Sized> (pub Mutex<T>);
type Amut<T> = Arc<Mut<T>>;

impl<T> Mut<T> {
    pub fn amut(value: T) -> Amut<T> {
        Arc::new(Mut(Mutex::new(value)))
    }
}

impl<T: PartialEq> PartialEq for Mut<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.0.lock().unwrap() == *other.0.lock().unwrap()
    }
}
impl<T: Eq> Eq for Mut<T> {}

pub trait Walk {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool);
}

#[derive(Debug)]
pub enum UDump {
    Str(String),
    Op(String, Vec<Box<UDump>>),
}

pub trait Dump: Walk {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump;
}

pub type Signal = (f64, f64);

pub trait Unit: Dump {
    fn proc(&mut self, time: &Time) -> Signal;
}

pub trait Osc: Unit {
    fn set_freq(&mut self, freq: AUnit);
}

pub enum ADSR {
    Attack,
    Decay,
    Sustin,
    Release,
    None,
}

pub trait Eg: Unit {
    fn set_state(&mut self, state: ADSR, eplaced: u64);
}

pub struct Table(pub Vec<f64>);

impl Table {
    pub fn new(vec: Vec<f64>) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Tab(Mut::amut(Table(vec)))))
    }
}

impl Walk for Table {
    fn walk(&self, _f: &mut FnMut(&AUnit) -> bool) {}
}

impl Dump for Table {
    fn dump(&self, _shared_vec: &Vec<AUnit>, _shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        for v in self.0.iter() {
            vec.push(Box::new(UDump::Str(v.to_string())));
        }
        UDump::Op("table".to_string(), vec)
    }
}

impl Unit for Table {
    fn proc(&mut self, _time: &Time) -> Signal {  // dummy
        (0.0, 0.0)
    }
}

pub struct Pattern(pub Vec<Box<Message>>);

impl Pattern {
    pub fn new(vec: Vec<Box<Message>>) -> AUnit {
        Mut::amut(UnitGraph::new(Node::Pat(Mut::amut(Pattern(vec)))))
    }
}

impl Walk for Pattern {
    fn walk(&self, _f: &mut FnMut(&AUnit) -> bool) {}
}

impl Dump for Pattern {
    fn dump(&self, _shared_vec: &Vec<AUnit>, _shared_map: &HashMap<usize, String>) -> UDump {
        let mut vec = Vec::new();
        let m = super::super::time::Measure { beat: 4, note: 4 };

        for ev in self.0.iter() {
            match &**ev {
                Message::Note(pitch, len) => {
                    let pitch_s = to_str(&pitch);
                    let len_s = to_len(&len, &m);
                    vec.push(Box::new(UDump::Str(format!("({} {})",  pitch_s, len_s))));
                },
                Message::Loop => vec.push(Box::new(UDump::Str("loop".to_string()))),
            }
        }
        UDump::Op("pat".to_string(), vec)
    }
}

impl Unit for Pattern {
    fn proc(&mut self, _time: &Time) -> Signal {  // dummy
        (0.0, 0.0)
    }
}

pub enum Node {
    Val(f64),
    Sig(Amut<Unit + Send>),
    Osc(Amut<Osc + Send>),
    Eg(Amut<Eg + Send>),
    Tab(Amut<Table>),
    Pat(Amut<Pattern>),
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Node::Val(s) => match other {
                Node::Val(o) => s == o,
                _ => false,
            },
            Node::Sig(s) => match other {
                Node::Sig(o) => Arc::ptr_eq(&s, &o),
                _ => false,
            },
            Node::Osc(s) => match other {
                Node::Osc(o) => Arc::ptr_eq(&s, &o),
                _ => false,
            },
            Node::Eg(s) => match other {
                Node::Eg(o) => Arc::ptr_eq(&s, &o),
                _ => false,
            },
            Node::Tab(s) => match other {
                Node::Tab(o) => Arc::ptr_eq(&s, &o),
                _ => false,
            },
            Node::Pat(s) => match other {
                Node::Pat(o) => Arc::ptr_eq(&s, &o),
                _ => false,
            },
        }
    }
}
impl Eq for Node {}

pub struct UnitGraph {
    pub last_tick: u64,
    pub last_sig: Signal,
    pub node: Node,
}

pub type AUnit = Amut<UnitGraph>;

impl PartialEq for UnitGraph {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl Eq for UnitGraph {}

impl UnitGraph {
    pub fn new(node: Node) -> UnitGraph {
        UnitGraph {
            last_tick: 0,
            last_sig: (0.0, 0.0),
            node: node,
        }
    }
}

impl Walk for Node {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        match self {
            Node::Val(_) => (),
            Node::Sig(u) => u.0.lock().unwrap().walk(f),
            Node::Osc(u) => u.0.lock().unwrap().walk(f),
            Node::Eg(u) => u.0.lock().unwrap().walk(f),
            Node::Tab(_) => (),
            Node::Pat(_) => (),
        }
    }
}

impl Dump for Node {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        match self {
            Node::Val(v) => UDump::Str(v.to_string()),
            Node::Sig(u) => u.0.lock().unwrap().dump(shared_vec, shared_map),
            Node::Osc(u) => u.0.lock().unwrap().dump(shared_vec, shared_map),
            Node::Eg(u) => u.0.lock().unwrap().dump(shared_vec, shared_map),
            Node::Tab(t) => t.0.lock().unwrap().dump(shared_vec, shared_map),
            Node::Pat(p) => p.0.lock().unwrap().dump(shared_vec, shared_map),
         }
    }
}

impl Unit for Node {
    fn proc(&mut self, time: &Time) -> Signal {
        match self {
            Node::Val(v) => (*v, *v),
            Node::Sig(u) => u.0.lock().unwrap().proc(time),
            Node::Osc(u) => u.0.lock().unwrap().proc(time),
            Node::Eg(u) => u.0.lock().unwrap().proc(time),
            Node::Tab(_) => (0.0, 0.0),
            Node::Pat(_) => (0.0, 0.0),
        }
    }
}

impl Osc for Node {
    fn set_freq(&mut self, freq: Amut<UnitGraph>) {
        match self {
            Node::Osc(u) => u.0.lock().unwrap().set_freq(freq),
            _ => (),
        }
    }
}

impl Eg for Node {
    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        match self {
            Node::Eg(u) => u.0.lock().unwrap().set_state(state, eplaced),
            _ => (),
        }
    }
}

impl Unit for UnitGraph {
    fn proc(&mut self, time: &Time) -> Signal {
        if self.last_tick < time.tick {
            self.last_sig = self.node.proc(&time);
            self.last_tick = time.tick;
            self.last_sig
        } else {
            self.last_sig
        }
    }
}

impl Walk for UnitGraph {
    fn walk(&self, f: &mut FnMut(&AUnit) -> bool) {
        self.node.walk(f);
    }
}

impl Dump for UnitGraph {
    fn dump(&self, shared_vec: &Vec<AUnit>, shared_map: &HashMap<usize, String>) -> UDump {
        self.node.dump(shared_vec, shared_map)
    }
}
