use std::sync::{Arc, Mutex};


use super::super::time::{Pos, Measure, PosOps, Time};
use super::super::event::{Event, Freq, Note, to_note, to_freq, to_pos};

use super::super::units::unit::{AUnit, UType, UnitGraph};

use super::super::tapirlisp::types::{Cons, Value, EvalError};
use super::super::tapirlisp::{to_vec, make_unit, print};

fn make_event(e: &Cons, pos: &mut Pos) -> Result<Vec<Box<Event>>, EvalError> {
    let mut ev = Vec::new();
    let time = Time { // TODO: read from global settings
        sample_rate: 0, tick: 0, bpm: 0.0,  // not used
        pos: Pos { bar: 0, beat: 0, pos: 0.0 },  // not used
        measure: Measure { beat: 4, note: 4 }
    };

    match e {
        Cons::Cons(name, cdr) => {
            if let Cons::Symbol(n) = &**name {
                if let Cons::Cons(len, _) = &**cdr {
                    let len = match &**len {
                        Cons::Number(l) => to_pos(*l as u32),
                        _ => to_pos(4),
                    };
                    match to_note(&n) {
                        Note::Rest => {
                            *pos = pos.add(len, &time);
                        },
                        n => {
                            ev.push(Box::new(Event::On(pos.clone(), to_freq(&n))));
                            *pos = pos.add(len, &time);
                            ev.push(Box::new(Event::Off(pos.clone())));
                        },
                    }
                } else {
                    // without length
                }
            } else {
                panic!("invalid length: {:?}", print(e));
            }
        },
        Cons::Symbol(name) => {
            match &name[..] {
                "loop" => ev.push(Box::new(Event::Loop(pos.clone()))),
                _ => panic!("unknown keyword or not implemented: {:?}", pos),
            }
        },
        sexp => {
            panic!("{:?} is not valid event", print(sexp));
        },
    }
    Ok(ev)
}

fn eval_events(events: Vec<Box<Cons>>) -> Vec<Box<Event>> {
    let mut ev: Vec<Box<Event>> = Vec::new();
    let mut pos = Pos { bar: 0, beat: 0, pos: 0.0 };
    for e in events.iter() {
        match &mut make_event(e, &mut pos) {
            Ok(vec) => ev.append(vec),
            err => panic!("aaaaaaaaaaaaaaaaaaaaaaaa"),
        }
    }
    ev
}

fn eval_call(name: &Cons, args: &Cons) -> Result<Value, EvalError> {
    match name {
        Cons::Symbol(name) if &name[..] == "pat" => {
            let vec = to_vec(&args);
            if vec.len() == 1 {
                Ok(Value::Pattern(eval_events(to_vec(&vec[0]))))
            } else {
                Err(EvalError::FnWrongParams("pat".to_string(), vec))
            }
        },
        Cons::Symbol(name) => {
            match make_unit(&name, to_vec(&args)) {
                Ok(u) => Ok(Value::Unit(u)),
                Err(err) => Err(err),
            }
        }
        c => Err(EvalError::FnMalformedName(Box::new(c.clone()))),
    }
}

pub fn eval(sexp: &Cons) -> Result<Value, EvalError> {
    match sexp {
        Cons::Cons(car, cdr) => eval_call(car, cdr),
        Cons::Symbol(name) => Err(EvalError::TodoSearchValueFromBinding),
        Cons::Number(num) => Ok(Value::Unit(Arc::new(Mutex::new(UnitGraph::Value(*num))))),
        Cons::Nil => Err(EvalError::Nil),
    }
}
