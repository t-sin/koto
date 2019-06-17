use std::sync::{Arc, Mutex};

use super::super::time::{Pos, Measure, PosOps, Time};
use super::super::event::{Event, Note, to_note, to_freq, to_pos};

use super::super::units::unit::{AUnit, UType, UnitGraph};
use super::super::units::core::{Pan, Offset, Gain, Add, Multiply};

use super::super::units::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};

use super::super::tapirlisp::{print, eval};
use super::super::tapirlisp::types::{Cons, Value, EvalError};

// core units

fn make_pan(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Arc::new(Mutex::new(
            UnitGraph::Unit(UType::Sig(
                Arc::new(Mutex::new(Pan {
                    v: match &*args[0] {
                        Cons::Number(n) => Arc::new(Mutex::new(UnitGraph::Value(*n))),
                        exp => match eval(&exp) {
                            Ok(Value::Unit(unit)) => unit,
                            Ok(Value::Pattern(p)) => return Err(EvalError::NotAUnit(p)),
                            Err(err) => return Err(err),
                        }
                    },
                    src: match eval(&args[1]) {
                        Ok(Value::Unit(src)) => src,
                        Ok(Value::Pattern(p)) => return Err(EvalError::NotAUnit(p)),
                        Err(err) => return Err(err),
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
                        exp => return Err(EvalError::NotANumber(print(&exp))),
                    },
                    src: match eval(&args[1]) {
                        Ok(Value::Unit(src)) => src,
                        Ok(Value::Pattern(p)) => return Err(EvalError::NotAUnit(p)),
                        Err(err) => return Err(err),
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
                        exp => return Err(EvalError::NotANumber(print(&exp))),
                    },
                    src: match eval(&args[1]) {
                        Ok(Value::Unit(src)) => src,
                        Ok(Value::Pattern(p)) => return Err(EvalError::NotAUnit(p)),
                        Err(err) => return Err(err),
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
                            Ok(Value::Unit(unit)) => v.push(unit),
                            Ok(Value::Pattern(p)) => return Err(EvalError::NotAUnit(p)),
                            Err(err) => return Err(err),
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
                            Ok(Value::Unit(unit)) => v.push(unit),
                            Ok(Value::Pattern(p)) => return Err(EvalError::NotAUnit(p)),
                            Err(err) => return Err(err),
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
            Ok(Value::Unit(unit)) => if let UnitGraph::Value(v) = *unit.lock().unwrap() {
                Ok(Rand::new(v as u64))
            } else {
                Ok(Rand::new(0))
            },
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

fn make_sine(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1]) {
                Ok(Value::Unit(freq)) => Ok(Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Sine {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                        }))
                    ))
                ))),
                Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
                Err(err) => Err(err),
            },
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("sine"), args))
    }
}

fn make_tri(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1]) {
                Ok(Value::Unit(freq)) => Ok(Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Tri {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                        }))
                    ))
                ))),
                Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
                Err(err) => Err(err),
            },
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
}

fn make_saw(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1]) {
                Ok(Value::Unit(freq)) => Ok(Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Saw {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                        }))
                    ))
                ))),
                Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
                Err(err) => Err(err),
            },
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
 }

fn make_pulse(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 3 {
        match eval(&args[0]) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1]) {
                Ok(Value::Unit(freq)) => match eval(&args[2]) {
                    Ok(Value::Unit(duty)) => Ok(Arc::new(Mutex::new(
                        UnitGraph::Unit(UType::Osc(
                            Arc::new(Mutex::new(Pulse {
                                init_ph: init_ph,
                                ph: 0.0,
                                freq: freq,
                                duty: duty,
                            }))
                        ))
                    ))),
                    Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
                    Err(err) => Err(err),
                },
                Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
                Err(err) => Err(err),
            },
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("pulse"), args))
    }
}

// wavetable oscillator

fn make_phase(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        match eval(&args[0]) {
            Ok(Value::Unit(osc)) => Ok(Phase::new(osc)),
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("phase"), args))
    }
}

fn make_wavetable(args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0]) {
            Ok(Value::Unit(table)) => match eval(&args[1]) {
                Ok(Value::Unit(osc)) => Ok(WaveTable::new(table, osc)),
                Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
                Err(err) => Err(err),
            },
            Ok(Value::Pattern(p)) => Err(EvalError::NotAUnit(p)),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

pub fn make_unit(name: &str, args: Vec<Box<Cons>>) -> Result<AUnit, EvalError> {
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

pub fn make_event(e: &Cons, pos: &mut Pos) -> Result<Vec<Box<Event>>, EvalError> {
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
                return Err(EvalError::EvWrongParams(print(e)))
            }
        },
        Cons::Symbol(name) => {
            match &name[..] {
                "loop" => ev.push(Box::new(Event::Loop(pos.clone()))),
                name => return Err(EvalError::EvUnknown(name.to_string())),
            }
        },
        sexp => {
            return Err(EvalError::EvMalformedEvent(print(sexp)))
        },
    }
    Ok(ev)
}
