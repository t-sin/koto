use super::super::tapirlisp as lisp;
use super::super::tapirlisp::cons::Cons;

use super::super::time::{Pos, Measure, PosOps, Time};

use super::event::{Event, Freq, Note, to_note, to_freq, to_pos};

fn eval_event(e: &Cons, pos: &mut Pos) -> Vec<Box<Event>> {
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
                panic!("invalid length: {:?}", lisp::print(e));
            }
        },
        Cons::Symbol(name) => {
            match &name[..] {
                "loop" => ev.push(Box::new(Event::Loop(pos.clone()))),
                _ => panic!("unknown keyword or not implemented: {:?}", pos),
            }
        },
        sexp => {
            panic!("{:?} is not valid event", lisp::print(sexp));
        },
    }
    ev
}

fn eval_events(head: &Cons, rest: &Cons, pos: &mut Pos) -> Vec<Box<Event>> {
    let mut ev: Vec<Box<Event>> = Vec::new();
    ev.append(&mut eval_event(head, pos));
    match rest {
        Cons::Cons(h, r) => ev.append(&mut eval_events(h, r, pos)),
        _ => (),
    }
    ev
}

pub fn eval_one(sexp: &Cons) -> Vec<Box<Event>> {
    match sexp {
        Cons::Cons(car, cdr) => {
            let mut pos = Pos { bar: 0, beat: 0, pos: 0.0 };
            eval_events(car, cdr, &mut pos)
        },
        Cons::Symbol(s) => {
            println!("symbol is not allowed here: {:?}", s);
            Vec::new()
        },
        Cons::Number(n) => {
            println!("number is not allowed here: {:?}", n);
            Vec::new()
        },
        Cons::Nil => panic!("empty list"),
    }
}
