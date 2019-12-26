use std::collections::VecDeque;

use super::super::sexp::{Cons, print, to_vec};
use super::super::event::{Message, to_note, to_pos};

use super::super::ugen::core::{UG, UGen, Aug, Proc, Osc, Eg, Table, Pattern};
use super::super::ugen::misc::{Pan, Clip, Offset, Gain, Add, Multiply, Out};
use super::super::ugen::osc::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};
use super::super::ugen::seq::{Trigger, AdsrEg}; //, Seq};
// use super::super::units::effect::{LPFilter, Delay};

use super::types::{Value, Env, EvalError};

// core units

fn make_pan(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match eval(&args[1], env) {
            Ok(Value::Unit(src)) => match &*args[0] {
                Cons::Number(n) => Ok(Pan::new(Aug::val(*n), src)),
                exp => match eval(&exp, env) {
                    Ok(Value::Unit(v)) => Ok(Pan::new(v, src)),
                    Ok(_v) => Err(EvalError::NotAug),
                    Err(err) => Err(err),
                },
            },
            Ok(_v) => return Err(EvalError::NotAug),
            Err(err) => return Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("pan"), args))
    }
 }

fn make_clip(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 3 {
        match &*args[0] {
            Cons::Number(min) => match &*args[1] {
                Cons::Number(max) => match eval(&args[2], env) {
                    Ok(Value::Unit(src)) => Ok(Clip::new(*min, *max, src)),
                    Ok(_v) => Err(EvalError::NotAug),
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

fn make_offset(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match &*args[0] {
            Cons::Number(n) => match eval(&args[1], env) {
                Ok(Value::Unit(src)) => Ok(Offset::new(*n, src)),
                Ok(_v) => return Err(EvalError::NotAug),
                Err(err) => return Err(err),
            },
            exp => return Err(EvalError::NotANumber(print(&exp))),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("offset"), args))
    }
}

fn make_gain(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match &*args[0] {
            Cons::Number(n) => match eval(&args[1], env) {
                Ok(Value::Unit(src)) => Ok(Gain::new(*n, src)),
                Ok(_v) => return Err(EvalError::NotAug),
                Err(err) => return Err(err),
            },
            exp => return Err(EvalError::NotANumber(print(&exp))),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("gain"), args))
    }
}

fn make_add(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    let mut v: Vec<Aug> = Vec::new();
    for s in args.iter() {
        match eval(s, env) {
            Ok(Value::Unit(unit)) => v.push(unit),
            Ok(_v) => return Err(EvalError::NotAug),
            Err(err) => return Err(err),
        }
    }
    Ok(Add::new(v))
}

fn make_multiply(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    let mut v: Vec<Aug> = Vec::new();
    for s in args.iter() {
        match eval(s, env) {
            Ok(Value::Unit(unit)) => v.push(unit),
            Ok(_v) => return Err(EvalError::NotAug),
            Err(err) => return Err(err),
        }
    }
    Ok(Multiply::new(v))
}

// oscillators

fn make_rand(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 1 {
        match eval(&args[0], env) {
            Ok(Value::Unit(unit)) => if let UG::Val(v) = unit.0.lock().unwrap().ug {
                Ok(Rand::new(v as u64))
            } else {
                Ok(Rand::new(0))
            },
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

fn make_sine(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => Ok(Sine::new(init_ph, freq)),
                Ok(_v) => Err(EvalError::NotAug),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("sine"), args))
    }
}

fn make_tri(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => Ok(Tri::new(init_ph, freq)),
                Ok(_v) => Err(EvalError::NotAug),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("tri"), args))
    }
}

fn make_saw(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => Ok(Saw::new(init_ph, freq)),
                Ok(_v) => Err(EvalError::NotAug),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("saw"), args))
    }
 }

fn make_pulse(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 3 {
        match eval(&args[0], env) {
            Ok(Value::Unit(init_ph)) => match eval(&args[1], env) {
                Ok(Value::Unit(freq)) => match eval(&args[2], env) {
                    Ok(Value::Unit(duty)) => Ok(Pulse::new(init_ph, freq, duty)),
                    Ok(_v) => Err(EvalError::NotAug),
                    Err(err) => Err(err),
                },
                Ok(_v) => Err(EvalError::NotAug),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("pulse"), args))
    }
}

// wavetable oscillator

fn make_table(args: Vec<Box<Cons>>, _env: &mut Env) -> Result<Aug, EvalError> {
    let mut table = Vec::new();
    if args.len() > 0 {
        for s in args.iter() {
            match **s {
                Cons::Number(n) => table.push(n),
                _ => return Err(EvalError::NotANumber(print(s))),
            }
        }
        Ok(Aug::new(UGen::new(UG::Tab(Table::new(table)))))
    } else {
        Err(EvalError::FnWrongParams("table".to_string(), args))
    }

}

fn make_phase(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 1 {
        match eval(&args[0], env) {
            Ok(Value::Unit(osc)) => Ok(Phase::new(osc)),
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("phase"), args))
    }
}

fn make_wavetable(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 2 {
        match eval(&args[1], env) {
            Ok(Value::Unit(ph)) => match eval(&args[0], env) {
                Ok(Value::Unit(table)) => {
                    let mut node_type = 0;
                    match &table.0.lock().unwrap().ug {
                        UG::Osc(_) => node_type = 1,
                        UG::Tab(_) => node_type = 2,
                        _ => (),
                    };
                    match node_type {
                        1 => Ok(WaveTable::from_osc(table.clone(), ph, &env.time)),
                        2 => Ok(WaveTable::from_table(table.clone(), ph)),
                        _ => return Err(EvalError::NotAug),
                    }
                },
                Ok(_v) => Err(EvalError::NotAug),
                Err(err) => Err(err),
            },
            Ok(_v) => Err(EvalError::NotAug),
            Err(err) => Err(err),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("wavetable"), args))
    }
}

// // sequencer

// pub fn make_msg(e: &Cons, _env: &mut Env) -> Result<Vec<Box<Message>>, EvalError> {
//     let mut ev = Vec::new();
//     match e {
//         Cons::Cons(name, cdr) => {
//             if let Cons::Symbol(pitch) = &**name {
//                 if let Cons::Cons(len, _) = &**cdr {
//                     let len = match &**len {
//                         Cons::Number(l) => to_pos(*l as u32),
//                         _ => to_pos(4),
//                     };
//                     ev.push(Box::new(Message::Note(to_note(pitch), len)));
//                 } else {
//                     // without length
//                 }
//             } else {
//                 return Err(EvalError::EvWrongParams(print(e)))
//             }
//         },
//         Cons::Symbol(name) => {
//             match &name[..] {
//                 "loop" => ev.push(Box::new(Message::Loop)),
//                 name => return Err(EvalError::EvUnknown(name.to_string())),
//             }
//         },
//         sexp => {
//             return Err(EvalError::EvMalformedEvent(print(sexp)))
//         },
//     }
//     Ok(ev)
// }

// fn eval_msgs(events: Vec<Box<Cons>>, env: &mut Env) -> Result<Vec<Box<Message>>, EvalError> {
//     let mut ev: Vec<Box<Message>> = Vec::new();
//     for e in events.iter() {
//         match &mut make_msg(e, env) {
//             Ok(vec) => ev.append(vec),
//             Err(err) => return Err(err.clone()),
//         }
//     }
//     Ok(ev)
// }

// fn make_pat(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
//     match eval_msgs(args, env) {
//         Ok(msgs) => Ok(Pattern::new(msgs)),
//         Err(err) => Err(err),
//     }
// }

fn make_trig(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() > 0 {
        // TODO: implement `(trig $eg $egs1 $egs2 ...)`
        let mut egs = Vec::<Aug>::new();
        for exp in &args[1..] {
            match eval(exp, env) {
                Ok(Value::Unit(eg)) => egs.push(eg),
                Ok(_v) => return Err(EvalError::NotAug),
                _err => return Err(EvalError::FnWrongParams(String::from("trig"), args)),
            }
        }
        match eval(&args[0], env) {
            Ok(Value::Unit(eg)) => Ok(Trigger::new(eg, egs)),
            Ok(_v) => Err(EvalError::NotAug),
            _err => Err(EvalError::FnWrongParams(String::from("trig"), args)),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("trig"), args))
    }
}

fn make_adsr_eg(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() == 4 {
        match eval(&args[0], env) {
            Ok(Value::Unit(a)) => match eval(&args[1], env) {
                Ok(Value::Unit(d)) => match eval(&args[2], env) {
                    Ok(Value::Unit(s)) => match eval(&args[3], env) {
                        Ok(Value::Unit(r)) => Ok(AdsrEg::new(a.clone(), d, s, r)),
                        Ok(_v) => Err(EvalError::NotAug),
                        _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
                    },
                    Ok(_v) => Err(EvalError::NotAug),
                    _err => Err(EvalError::FnWrongParams(String::from("adsr"), args))
                },
                Ok(_v) => Err(EvalError::NotAug),
                _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
            },
            Ok(_v) => Err(EvalError::NotAug),
            _err => Err(EvalError::FnWrongParams(String::from("adsr"), args)),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("asdr"), args))
    }
}

// fn make_seq(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
//     if args.len() == 3 {
//         match eval(&args[1], env) {
//             Ok(Value::Unit(osc)) => match eval(&args[2], env) {
//                 Ok(Value::Unit(eg)) => match eval(&args[0], env) {
//                     Ok(Value::Unit(pat)) => Ok(Seq::new(pat, osc, eg, &env.time)),
//                     _ => Err(EvalError::NotAPattern),
//                 },
//                 Ok(_v) => Err(EvalError::NotAug),
//                 Err(err) => Err(err),
//             },
//             Ok(_v) => Err(EvalError::NotAug),
//             Err(err) => Err(err),
//         }
//     } else {
//         Err(EvalError::FnWrongParams(String::from("seq"), args))
//     }
// }

// // effects

// fn make_lpf(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
//     if args.len() == 3 {
//         match eval(&args[0], env) {
//             Ok(Value::Unit(freq)) => match eval(&args[1], env) {
//                 Ok(Value::Unit(q)) => match eval(&args[2], env) {
//                     Ok(Value::Unit(src)) => Ok(LPFilter::new(freq, q, src)),
//                     _ => Err(EvalError::NotAPattern),
//                 },
//                 Ok(_v) => Err(EvalError::NotAug),
//                 Err(err) => Err(err),
//             },
//             Ok(_v) => Err(EvalError::NotAug),
//             Err(err) => Err(err),
//         }
//     } else {
//         Err(EvalError::FnWrongParams(String::from("lpf"), args))
//     }
// }


// fn make_delay(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
//     if args.len() == 4 {
//         match eval(&args[0], env) {
//             Ok(Value::Unit(time)) => match eval(&args[1], env) {
//                 Ok(Value::Unit(feedback)) => match eval(&args[2], env) {
//                     Ok(Value::Unit(mix)) => match eval(&args[3], env) {
//                         Ok(Value::Unit(src)) => Ok(Delay::new(time, feedback, mix, src, env)),
//                         Ok(_v) => Err(EvalError::NotAug),
//                         Err(_err) => Err(EvalError::NotAug),
//                     }
//                     Ok(_v) => Err(EvalError::NotAug),
//                     Err(err) => Err(err),
//                 },
//                 Ok(_v) => Err(EvalError::NotAug),
//                 Err(err) => Err(err),
//             },
//             Ok(_v) => Err(EvalError::NotAug),
//             Err(err) => Err(err),
//         }
//     } else {
//         Err(EvalError::FnWrongParams(String::from("delay"), args))
//     }
// }

// utility

fn make_out(args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
    if args.len() >= 1 {
        match(*args[0]) {
            Cons::Number(vol) => {
                let mut v: Vec<Aug> = Vec::new();
                for s in args[1..].iter() {
                    match eval(s, env) {
                        Ok(Value::Unit(unit)) => v.push(unit),
                        Ok(_v) => return Err(EvalError::NotAug),
                        Err(err) => return Err(err),
                    }
                }
                Ok(Out::new(vol, v))
            },
            _ => Err(EvalError::NotANumber(print(&args[0]))),
        }
    } else {
        Err(EvalError::FnWrongParams(String::from("out"), args))
    }
}

pub fn make_unit(name: &str, args: Vec<Box<Cons>>, env: &mut Env) -> Result<Aug, EvalError> {
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
        // "table" => make_table(args, env),
        "phase" => make_phase(args, env),
        "wavetable" => make_wavetable(args, env),
        // // sequencer
        // "pat" => make_pat(args, env),
        "trig" => make_trig(args, env),
        "adsr" => make_adsr_eg(args, env),
        // "seq" => make_seq(args, env),
        // // fx
        // "lpf" => make_lpf(args, env),
        // "delay" => make_delay(args, env),
        // // for convinience
        "out" => make_out(args, env),
        _ => Err(EvalError::FnUnknown(String::from(name))),
    }
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
        Cons::Symbol(name) if &name[..] == "bpm" => {
            let vec = to_vec(&args);
            if vec.len() == 1 {
                match *vec[0] {
                    Cons::Number(n) => {
                        env.time.bpm = n;
                        Ok(Value::Nil)
                    },
                    _ => Err(EvalError::NotANumber(print(&vec[0]))),
                }
            } else {
                Err(EvalError::FnWrongParams("bpm".to_string(), vec))
            }
        },
        Cons::Symbol(name) if &name[..] == "measure" => {
            let vec = to_vec(&args);
            if vec.len() == 2 {
                match *vec[0] {
                    Cons::Number(beat) => match *vec[1] {
                        Cons::Number(note) => {
                            env.time.measure.beat = beat as u64;
                            env.time.measure.note = note as u64;
                            Ok(Value::Nil)
                        },
                        _ => Err(EvalError::NotANumber(print(&vec[1]))),
                    },
                    _ => Err(EvalError::NotANumber(print(&vec[0]))),
                }
            } else {
                Err(EvalError::FnWrongParams("measure".to_string(), vec))
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
        Cons::Number(num) => Ok(Value::Unit(Aug::val(*num))),
        Cons::Nil => Ok(Value::Nil),
    }
}

pub fn eval_all(sexp_vec: Vec<Box<Cons>>, env: &mut Env) -> Result<Value, EvalError> {
    let mut q = VecDeque::new();
    for sexp in sexp_vec.iter() {
        match eval(sexp, env) {
            Ok(v) => q.push_back(v),
            Err(err) => return Err(err),
        }
    }
    match q.len() {
        0 => Ok(Value::Nil),
        _ => Ok(q.pop_back().unwrap()),
    }

}
