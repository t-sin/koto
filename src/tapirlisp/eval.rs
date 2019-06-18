use std::sync::{Arc, Mutex};

use super::super::time::Pos;
use super::super::event::Event;

use super::super::units::unit::{Node, UnitGraph};

use super::super::tapirlisp::types::{Cons, Value, Env, EvalError};
use super::super::tapirlisp::to_vec;
use super::super::tapirlisp::make::{make_unit, make_event};

fn eval_events(events: Vec<Box<Cons>>, env: &mut Env) -> Result<Vec<Box<Event>>, EvalError> {
    let mut ev: Vec<Box<Event>> = Vec::new();
    let mut pos = Pos { bar: 0, beat: 0, pos: 0.0 };
    for e in events.iter() {
        match &mut make_event(e, &mut pos, env) {
            Ok(vec) => ev.append(vec),
            Err(err) => return Err(err.clone()),
        }
    }
    Ok(ev)
}

fn eval_def(name: &Cons, sexp: &Cons, env: &mut Env) -> Result<Value, EvalError> {
    match name {
        Cons::Symbol(name) => {
            if env.binding.contains_key(name) {
                Err(EvalError::AlreadyBound(name.to_string()))
            } else {
                match eval(sexp, env) {
                    Ok(v) => {
                        env.binding.insert(name.to_string(), Box::new(v));
                        Ok(Value::Nil)
                    },
                    Err(err) => Err(err),
                }
            }
        },
        exp => Err(EvalError::NotASymbol(Box::new(exp.clone()))),
    }
}

fn eval_call(name: &Cons, args: &Cons, env: &mut Env) -> Result<Value, EvalError> {
    match name {
        Cons::Symbol(name) if &name[..] == "pat" => {
            let vec = to_vec(&args);
            if vec.len() == 1 {
                match eval_events(to_vec(&vec[0]), env) {
                    Ok(ev) => Ok(Value::Pattern(ev)),
                    Err(err) => Err(err),
                }
            } else {
                Err(EvalError::FnWrongParams("pat".to_string(), vec))
            }
        },
        Cons::Symbol(name) if &name[..] == "def" => {
            let vec = to_vec(&args);
            if vec.len() == 2 {
                match eval_def(&*vec[0], &*vec[1], env) {
                    Ok(v) => Ok(v),
                    Err(err) => Err(err),
                }
            } else {
                Err(EvalError::FnWrongParams("def".to_string(), vec))
            }
        },
        Cons::Symbol(name) => {
            match make_unit(&name, to_vec(&args), env) {
                Ok(u) => Ok(Value::Unit(u)),
                Err(err) => Err(err),
            }
        }
        c => Err(EvalError::FnMalformedName(Box::new(c.clone()))),
    }
}

pub fn eval(sexp: &Cons, env: &mut Env) -> Result<Value, EvalError> {
    match sexp {
        Cons::Cons(car, cdr) => eval_call(car, cdr, env),
        Cons::Symbol(name) => match env.binding.get(name) {
            Some(v) => Ok((**v).clone()),
            None => Err(EvalError::UnboundVariable(name.to_string())),
        }
        Cons::Number(num) => Ok(Value::Unit(
            Arc::new(Mutex::new(UnitGraph::new(Node::Val(*num)))))),
        Cons::Nil => Ok(Value::Nil),
    }
}
