use std::sync::{Arc, Mutex};

use super::super::tapirlisp as lisp;
use super::super::tapirlisp::types::{Cons, EvalError};

use super::unit::{AUnit, UType, UnitGraph};
use super::core::{Pan, Offset, Gain, Add, Multiply};

use super::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};

// core units

fn make_pan(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Sig(
                Arc::new(Mutex::new(Pan {
                    v: match &*args[0] {
                        Cons::Number(n) => Arc::new(Mutex::new(UnitGraph::Value(*n))),
                        exp => match eval(&exp) {
                            Ok(unit) => unit,
                            err => return err,
                        }
                    },
                    src: match eval(&args[1]) {
                        Ok(src) => src,
                        err => return err,
                    },
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
                    src: match eval(&args[1]) {
                        Ok(src) => src,
                        err => return err,
                    },
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
                    src: match eval(&args[1]) {
                        Ok(src) => src,
                        err => return err,
                    },
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
                    for s in args.iter() {
                        match eval(s) {
                            Ok(unit) => v.push(unit),
                            err => return err,
                        }
                    }
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
                    for s in args.iter() {
                        match eval(s) {
                            Ok(unit) => v.push(unit),
                            err => return err,
                        }
                    }
                    v
                }
            }))
        ))
    )))
}

// oscillators

fn make_rand(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        match eval(&args[0]) {
            Ok(unit) => if let UnitGraph::Value(v) = *unit.lock().unwrap() {
                Ok(Rand::new(v as u64))
            } else {
                Ok(Rand::new(0))
            },
            err => err,
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

fn make_sine(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(init_ph) => match eval(&args[1]) {
                Ok(freq) => Ok(Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Sine {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                        }))
                    ))
                ))),
                err => err,
            },
            err => err,
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("sine"), args))
    }
}

fn make_tri(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(init_ph) => match eval(&args[1]) {
                Ok(freq) => Ok(Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Tri {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                        }))
                    ))
                ))),
                err => err,
            },
            err => err,
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
}

fn make_saw(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(init_ph) => match eval(&args[1]) {
                Ok(freq) => Ok(Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Saw {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                        }))
                    ))
                ))),
                err => err,
            },
            err => err,
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
 }

fn make_pulse(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 3 {
        match eval(&args[0]) {
            Ok(init_ph) => match eval(&args[1]) {
                Ok(freq) => match eval(&args[2]) {
                    Ok(duty) => Ok(Arc::new(Mutex::new(
                        UnitGraph::Unit(UType::Osc(
                            Arc::new(Mutex::new(Pulse {
                                init_ph: init_ph,
                                ph: 0.0,
                                freq: freq,
                                duty: duty,
                            }))
                        ))
                    ))),
                    err => err,
                },
                err => err,
            },
            err => err,
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("pulse"), args))
    }
}

// wavetable oscillator

fn make_phase(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        match eval(&args[0]) {
            Ok(osc) => Ok(Phase::new(osc)),
            err => err,
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("phase"), args))
    }
}

fn make_wavetable(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(table) => match eval(&args[1]) {
                Ok(osc) => Ok(WaveTable::new(table, osc)),
                err => err,
            },
            err => err,
        }
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
