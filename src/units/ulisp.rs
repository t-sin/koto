use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use super::super::tapirlisp as lisp;
use super::super::tapirlisp::types::Cons;

use super::unit::{AUnit, UType, UnitGraph};
use super::core::{Pan, Offset, Gain, Add, Multiply};

use super::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};

#[derive(Debug)]
pub enum EvalError {
    FnWrongParams(String, Vec<Box<Cons>>),
    FnUnknown(String),
    FnMalformedName(Box<Cons>),
    NotANumber(String),
    TodoSearchValueFromBinding,
    Nil
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvalError::FnWrongParams(name, args) => {
                write!(f, "Wrong params for '{:?}' with args '{:?}'", name, args)
            },
            EvalError::FnUnknown(name) => {
                write!(f, "{:?} is unknown or not implemented.", name)
            },
            EvalError::FnMalformedName(cons) => {
                write!(f, "{:?} is not a symbol.", cons)
            },
            EvalError::NotANumber(s) => {
                write!(f, "{:?} is not a number", s)
            },
            EvalError::TodoSearchValueFromBinding => {
                write!(f, "TODO: searching from binding.")
            },
            EvalError::Nil => {
                write!(f, "nil.")
            },
        }
    }
}

impl Error for EvalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EvalError::FnWrongParams(_name, _args) => None,
            EvalError::FnUnknown(_) => None,
            EvalError::FnMalformedName(_) => None,
            EvalError::TodoSearchValueFromBinding => None,
            EvalError::NotANumber(_) => None,
            EvalError::Nil => None,
        }
    }
}

// core units

fn make_pan(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Sig(
                Arc::new(Mutex::new(Pan {
                    v: match &*args[0] {
                        Cons::Number(n) => Arc::new(Mutex::new(UnitGraph::Value(*n))),
                        exp => eval(&exp).unwrap(),
                    },
                    src: eval(&args[1]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("pan"), args))
    }
 }

fn make_offset(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Sig(
                Arc::new(Mutex::new(Offset {
                    v: match &*args[0] {
                        Cons::Number(n) => *n,
                        exp => return Err(EvalError::NotANumber(lisp::print(&exp))),
                    },
                    src: eval(&args[1]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("offset"), args))
    }
}

fn make_gain(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Sig(
                Arc::new(Mutex::new(Gain {
                    v: match &*args[0] {
                        Cons::Number(n) => *n,
                        exp => return Err(EvalError::NotANumber(lisp::print(&exp))),
                    },
                    src: eval(&args[1]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("gain"), args))
    }
}

fn make_add(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    Ok(Arc::new(Mutex::new(
        UnitGraph::Unit(UType::Sig(
            Arc::new(Mutex::new(Add {
                sources: {
                    let mut v: Vec<Arc<Mutex<UnitGraph>>> = Vec::new();
                    for s in args.iter() { v.push(eval(s).unwrap()) }
                    v
                }
            }))
        ))
    )))
}

fn make_multiply(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    Ok(Arc::new(Mutex::new(
        UnitGraph::Unit(UType::Sig(
            Arc::new(Mutex::new(Multiply {
                sources: {
                    let mut v: Vec<Arc<Mutex<UnitGraph>>> = Vec::new();
                    for s in args.iter() { v.push(eval(s).unwrap()) }
                    v
                }
            }))
        ))
    )))
}

// oscillators

fn make_rand(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        if let UnitGraph::Value(v) = *eval(&args[0]).unwrap().lock().unwrap() {
            Ok(Rand::new(v as u64))
        } else {
            Ok(Rand::new(0))
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

fn make_sine(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Osc(
                Arc::new(Mutex::new(Sine {
                    init_ph: eval(&args[0]).unwrap(),
                    ph: 0.0,
                    freq: eval(&args[1]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("sine"), args))
    }
}

fn make_tri(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Osc(
                Arc::new(Mutex::new(Tri {
                    init_ph: eval(&args[0]).unwrap(),
                    ph: 0.0,
                    freq: eval(&args[1]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
}

fn make_saw(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Osc(
                Arc::new(Mutex::new(Saw {
                    init_ph: eval(&args[0]).unwrap(),
                    ph: 0.0,
                    freq: eval(&args[1]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
 }

fn make_pulse(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 3 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Osc(
                Arc::new(Mutex::new(Pulse {
                    init_ph: eval(&args[0]).unwrap(),
                    ph: 0.0,
                    freq: eval(&args[1]).unwrap(),
                    duty: eval(&args[2]).unwrap(),
                }))
            ))
        )))
    } else {
        Err(EvalError::FnWrongParams(String::from("pulse"), args))
    }
}

// wavetable oscillator

fn make_phase(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        Ok(Phase::new(eval(&args[0]).unwrap()))
    } else {
        Err(EvalError::FnWrongParams(String::from("phase"), args))
    }
}

fn make_wavetable(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(WaveTable::new(eval(&args[0]).unwrap(), eval(&args[1]).unwrap()))
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

fn make_unit(name: &str, args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    match &name[..] {
        // core
        "pan" => make_pan(args),
        "offset" => make_offset(args),
        "gain" => make_gain(args),
        "+" => make_add(args),
        "*" => make_multiply(args),
        // oscillator
        "rand" => make_rand(args),
        "sine" => make_sine(args),
        "tri" => make_tri(args),
        "saw" => make_saw(args),
        "pulse" => make_pulse(args),
        "phase" => make_phase(args),
        "wavetable" => make_wavetable(args),
        _ => Err(EvalError::FnUnknown(String::from(name))),
    }
}

fn eval_call(name: &Cons, args: &Cons) -> Result<AUnit, EvalError> {
    match name {
        Cons::Symbol(n) => make_unit(&n[..], lisp::to_vec(&args)),
        c => Err(EvalError::FnMalformedName(Box::new(c.clone()))),
    }
}

pub fn eval(sexp: &Cons) -> Result<AUnit, EvalError> {
    match sexp {
        Cons::Cons(car, cdr) => eval_call(car, cdr),
        Cons::Symbol(name) => Err(EvalError::TodoSearchValueFromBinding),
        Cons::Number(num) => Ok(Arc::new(Mutex::new(UnitGraph::Value(*num)))),
        Cons::Nil => Err(EvalError::Nil),
    }
}

// TODO: unit graph serializer
