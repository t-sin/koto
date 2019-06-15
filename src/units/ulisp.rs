use std::sync::{Arc, Mutex};

use super::super::tapirlisp;
use super::super::tapirlisp::Cons;

use super::unit::{Amut, AUnit, UType, Osc, UnitGraph};
use super::core::{Pan, Offset, Gain, Add, Multiply};

use super::oscillator::{Sine, Tri, Saw, Pulse, Phase, WaveTable};

//// unit graph constructor (or eval?)

fn to_vec(list: &Cons) -> Vec<&Cons> {
    match list {
        Cons::Nil => Vec::new(),
        Cons::Cons(elem, rest) => {
            let mut v: Vec<&Cons> = Vec::new();
            v.push(elem);
            v.append(&mut to_vec(rest));
            v
        },
        _ => panic!("it's not proper list: {:?}", list),
    }
}

fn construct_one(name: &str, args: Vec<&Cons>) -> AUnit {
    match &name[..] {
        "sine" => {
            if args.len() == 2 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Sine {
                            init_ph: eval_one(args[0]),
                            ph: 0.0,
                            freq: eval_one(args[1]),
                    }))))))
            } else {
                panic!("wrong params");
            }
        },
        "tri" => {
            if args.len() == 2 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Tri {
                            init_ph: eval_one(args[0]),
                            ph: 0.0,
                            freq: eval_one(args[1]),
                }))))))
            } else {
                panic!("wrong params");
            }
        },
        "saw" => {
            if args.len() == 2 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Saw {
                            init_ph: eval_one(args[0]),
                            ph: 0.0,
                            freq: eval_one(args[1]),
                }))))))
            } else {
                panic!("wrong params");
            }
        },
        "pulse" => {
            if args.len() == 3 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Osc(
                        Arc::new(Mutex::new(Pulse {
                            init_ph: eval_one(args[0]),
                            ph: 0.0,
                            freq: eval_one(args[1]),
                            duty: eval_one(args[2]),
                }))))))
            } else {
                panic!("wrong params");
            }
        },
        "pan" => {
            if args.len() == 2 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Sig(
                        Arc::new(Mutex::new(Pan {
                            v: match args[0] {
                                Cons::Number(n) => Arc::new(Mutex::new(UnitGraph::Value(*n))),
                                exp => eval_one(exp),
                            },
                            src: eval_one(args[1]),
                }))))))
            } else {
                panic!("wrong params");
            }
        },
        "offset" => {
            if args.len() == 2 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Sig(
                        Arc::new(Mutex::new(Offset {
                            v: match args[0] {
                                Cons::Number(n) => *n,
                                exp => panic!("{:?} is not a number", tapirlisp::print(exp)),
                            },
                            src: eval_one(args[1]),
                }))))))
            } else {
                panic!("wrong params");
            }
        },
        "gain" => {
            if args.len() == 2 {
                Arc::new(Mutex::new(
                    UnitGraph::Unit(UType::Sig(
                        Arc::new(Mutex::new(Gain {
                            v: match args[0] {
                                Cons::Number(n) => *n,
                                exp => panic!("{:?} is not a number", tapirlisp::print(exp)),
                            },
                            src: eval_one(args[1]),
                }))))))
            } else {
                panic!("wrong params");
            }
        },
        "+" => {
             Arc::new(Mutex::new(
                 UnitGraph::Unit(UType::Sig(
                     Arc::new(Mutex::new(Add {
                         sources: {
                              let mut v: Vec<Arc<Mutex<UnitGraph>>> = Vec::new();
                              for s in args.iter() { v.push(eval_one(s)) }
                              v
                         }
                     }
             ))))))
        },
        "*" => {
             Arc::new(Mutex::new(
                 UnitGraph::Unit(UType::Sig(
                     Arc::new(Mutex::new(Multiply {
                         sources: {
                              let mut v: Vec<Arc<Mutex<UnitGraph>>> = Vec::new();
                              for s in args.iter() { v.push(eval_one(s)) }
                              v
                         }
                     }
             ))))))
        },
        "phase" => {
            if args.len() == 1 {
                Phase::new(eval_one(args[0]))
            } else {
                panic!("wrong params");
            }
        },
        "wavetable" => {
            if args.len() == 2 {
                WaveTable::new(eval_one(args[0]), eval_one(args[1]))
            } else {
                panic!("wrong params");
            }
        }
        _ => {
            println!("{:?} is unknown or not implemented.", name);
            Arc::new(Mutex::new(UnitGraph::Value(0.0)))
        },
    }
}

fn eval_list(name: &Cons, args: &Cons) -> AUnit {
    match name {
        Cons::Symbol(n) => construct_one(&n[..], to_vec(&args)),
        _ => panic!("ill formed form"),
    }
}

pub fn eval_one(sexp: &Cons) -> AUnit {
    match sexp {
        Cons::Cons(car, cdr) => eval_list(car, cdr),
        Cons::Symbol(name) => {
            println!("name: {:?}", name);
            Arc::new(Mutex::new(UnitGraph::Value(0.0)))
        },
        Cons::Number(num) => Arc::new(Mutex::new(UnitGraph::Value(*num))),
        Cons::Nil => panic!("what should I do?"),
    }
}

// TODO: unit graph serializer
