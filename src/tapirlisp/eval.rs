use std::sync::{Arc, Mutex};


use super::super::time::{Pos, Measure, PosOps, Time};
use super::super::event::{Event, Freq, Note, to_note, to_freq, to_pos};

use super::super::units::unit::{AUnit, UType, UnitGraph};

use super::super::tapirlisp::types::{Cons, Value, EvalError};
use super::super::tapirlisp::{to_vec, make_unit, make_event, print};

fn eval_events(events: Vec<Box<Cons>>) -> Result<Vec<Box<Event>>, EvalError> {
    let mut ev: Vec<Box<Event>> = Vec::new();
    let mut pos = Pos { bar: 0, beat: 0, pos: 0.0 };
    for e in events.iter() {
        match &mut make_event(e, &mut pos) {
            Ok(vec) => ev.append(vec),
            Err(err) => return Err(err.clone()),
        }
    }
    Ok(ev)
}

fn eval_call(name: &Cons, args: &Cons) -> Result<Value, EvalError> {
    match name {
        Cons::Symbol(name) if &name[..] == "pat" => {
            let vec = to_vec(&args);
            if vec.len() == 1 {
                match eval_events(to_vec(&vec[0])) {
                    Ok(ev) => Ok(Value::Pattern(ev)),
                    Err(err) => Err(err),
                }
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
