use super::super::time::{Pos, PosOps};
use super::super::event::{Event, Note, to_note, to_freq, to_pos};

use super::super::units::unit::{Mut, AUnit, Node, UnitGraph};
use super::super::units::core::{Pan, Clip, Offset, Gain, Add, Multiply};
use super::super::units::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};
use super::super::units::sequencer::{AdsrEg, Seq};

use super::super::tapirlisp::types::{Cons, Value, Env, EvalError};
use super::super::tapirlisp::rp::print;
use super::super::tapirlisp::to_vec;

// core units

fn make_pan(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Mut::amut(UnitGraph::new(Node::Sig(
                Mut::amut(Pan {
                v: match &*args[0] {
                    Cons::Number(n) => Mut::amut(UnitGraph::new(Node::Val(*n))),
                    exp => match eval(&exp, env) {
                        Ok(Value::Unit(unit)) => unit,
                        Ok(_v) => return Err(EvalError::NotAUnit),
                        Err(err) => return Err(err),
                    }
                },
                src: match eval(&args[1], env) {
                    Ok(Value::Unit(src)) => src,
                    Ok(_v) => return Err(EvalError::NotAUnit),
                    Err(err) => return Err(err),
                },
            })
        ))))
    } else {
        Err(EvalError::FnWrongParams(String::from("pan"), args))
    }
 }

fn make_clip(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 3 {
        match &*args[0] {
            Cons::Number(min) => match &*args[1] {
                Cons::Number(max) => match eval(&args[2], env) {
                    Ok(Value::Unit(src)) => Ok(Mut::amut(UnitGraph::new(Node::Sig(
                        Mut::amut(Clip {
                            min: *min, max: *max, src: src
                        })
                    )))),
                    Ok(_v) => Err(EvalError::NotAUnit),
                    Err(err) => Err(err),
                },
                exp => Err(EvalError::NotANumber(print(&exp))),
            },
            exp => Err(EvalError::NotANumber(print(&exp))),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("clip"), args))
    }
}

fn make_offset(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Offset {
                v: match &*args[0] {
                    Cons::Number(n) => *n,
                    exp => return Err(EvalError::NotANumber(print(&exp))),
                },
                src: match eval(&args[1], env) {
                    Ok(Value::Unit(src)) => src,
                    Ok(_v) => return Err(EvalError::NotAUnit),
                    Err(err) => return Err(err),
                },
            })
        ))))
    } else {
        Err(EvalError::FnWrongParams(String::from("offset"), args))
    }
}

fn make_gain(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        Ok(Mut::amut(UnitGraph::new(Node::Sig(
            Mut::amut(Gain {
                v: match &*args[0] {
                    Cons::Number(n) => *n,
                    exp => return Err(EvalError::NotANumber(print(&exp))),
                },
                src: match eval(&args[1], env) {
                    Ok(Value::Unit(src)) => src,
                    Ok(_v) => return Err(EvalError::NotAUnit),
                    Err(err) => return Err(err),
                },
            })
        ))))
    } else {
        Err(EvalError::FnWrongParams(String::from("gain"), args))
    }
}

fn make_add(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    Ok(Mut::amut(UnitGraph::new(Node::Sig(
        Mut::amut(Add {
            sources: {
                let mut v: Vec<AUnit> = Vec::new();
                for s in args.iter() {
                    match eval(s, env) {
                        Ok(Value::Unit(unit)) => v.push(unit),
                        Ok(_v) => return Err(EvalError::NotAUnit),
                        Err(err) => return Err(err),
                    }
                }
                v
            }
        })
    ))))
}

fn make_multiply(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    Ok(Mut::amut(UnitGraph::new(Node::Sig(
        Mut::amut(Multiply {
            sources: {
                let mut v: Vec<AUnit> = Vec::new();
                for s in args.iter() {
                    match eval(s, env) {
                        Ok(Value::Unit(unit)) => v.push(unit),
                        Ok(_v) => return Err(EvalError::NotAUnit),
                        Err(err) => return Err(err),
                    }
                }
                v
            }
        })
    ))))
}

// oscillators

fn make_rand(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        match eval(&args[0], env) {
            Ok(Value::Unit(unit)) => if let Node::Val(v) = unit.0.lock().unwrap().node {
                Ok(Rand::new(v as u64))
            } else {
                Ok(Rand::new(0))
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

fn make_sine(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => Ok(Mut::amut(
                    UnitGraph::new(Node::Osc(Mut::amut(Sine {
                        init_ph: init_ph,
                        ph: 0.0,
                        freq: freq,
                    })))
                )),
                Ok(_v) => Err(EvalError::NotAUnit),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("sine"), args))
    }
}

fn make_tri(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => Ok(Mut::amut(UnitGraph::new(Node::Osc(
                    Mut::amut(Tri {
                        init_ph: init_ph,
                        ph: 0.0,
                        freq: freq,
                    })
                )))),
                Ok(_v) => Err(EvalError::NotAUnit),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
}

fn make_saw(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => Ok(Mut::amut(
                    UnitGraph::new(Node::Osc(Mut::amut(Saw {
                        init_ph: init_ph,
                        ph: 0.0,
                        freq: freq,
                    })))
                )),
                Ok(_v) => Err(EvalError::NotAUnit),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
 }

fn make_pulse(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 3 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => match eval(&args[2], env) {
                    Ok(Value::Unit(duty)) => Ok(Mut::amut(UnitGraph::new(Node::Osc(
                        Mut::amut(Pulse {
                            init_ph: init_ph,
                            ph: 0.0,
                            freq: freq,
                            duty: duty,
                        })
                    )))),
                    Ok(_v) => Err(EvalError::NotAUnit),
                    Err(err) => Err(err),
                },
                Ok(_v) => Err(EvalError::NotAUnit),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("pulse"), args))
    }
}

// wavetable oscillator

fn make_phase(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 1 {
        match eval(&args[0], env) {
            Ok(Value::Unit(osc)) => Ok(Phase::new(osc)),
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("phase"), args))
    }
}

fn make_wavetable(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(table)) => match eval(&args[1], env) {
                Ok(Value::Unit(osc)) => Ok(WaveTable::new(table, osc)),
                Ok(_v) => Err(EvalError::NotAUnit),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

// sequencer

fn make_adsr_eg(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 4 {
        match eval(&args[0], env) {
            Ok(Value::Unit(a)) => match eval(&args[1], env) {
                Ok(Value::Unit(d)) => match eval(&args[2], env) {
                    Ok(Value::Unit(s)) => match eval(&args[3], env) {
                        Ok(Value::Unit(r)) => Ok(AdsrEg::new(a.clone(), d, s, r)),
                        Ok(_v) => Err(EvalError::NotAUnit),
                        _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
                    },
                    Ok(_v) => Err(EvalError::NotAUnit),
                    _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
                },
                Ok(_v) => Err(EvalError::NotAUnit),
                _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
            },
            Ok(_v) => Err(EvalError::NotAUnit),
            _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("asdr"), args))
    }
}

fn make_seq(args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    if args.len() == 3 {
        match eval(&args[0], env) {
            Ok(Value::Pattern(pat)) => match eval(&args[1], env) {
                Ok(Value::Unit(osc)) => match eval(&args[2], env) {
                    Ok(Value::Unit(eg)) => Ok(Seq::new(pat, osc, eg)),
                    Ok(_v) => Err(EvalError::NotAUnit),
                    Err(err) => Err(err),
                },
                Ok(_v) => Err(EvalError::NotAUnit),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAPattern),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("seq"), args))
    }
}

pub fn make_unit(name: &str, args: Vec<Box<Cons>>, env: &mut Env) -> Result<AUnit, EvalError> {
    match &name[..] {
        // core
        "pan" => make_pan(args, env),
        "clip" => make_clip(args, env),
        "offset" => make_offset(args, env),
        "gain" => make_gain(args, env),
        "+" => make_add(args, env),
        "*" => make_multiply(args, env),
        // oscillator
        "rand" => make_rand(args, env),
        "sine" => make_sine(args, env),
        "tri" => make_tri(args, env),
        "saw" => make_saw(args, env),
        "pulse" => make_pulse(args, env),
        "phase" => make_phase(args, env),
        "wavetable" => make_wavetable(args, env),
        // sequencer
        "adsr" => make_adsr_eg(args, env),
        "seq" => make_seq(args, env),
        _ => Err(EvalError::FnUnknown(String::from(name))),
    }
}

pub fn make_event(e: &Cons, pos: &mut Pos, env: &mut Env) -> Result<Vec<Box<Event>>, EvalError> {
    let mut ev = Vec::new();
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
                            *pos = pos.add(len, &env.time.measure);
                        },
                        n => {
                            ev.push(Box::new(Event::On(pos.clone(), to_freq(&n))));
                            *pos = pos.add(len, &env.time.measure);
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
            Mut::amut(UnitGraph::new(Node::Val(*num))))),
        Cons::Nil => Ok(Value::Nil),
    }
}
