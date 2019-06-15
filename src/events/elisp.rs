use super::super::tapirlisp as lisp;
use super::super::tapirlisp::Cons;

use super::super::time::{Pos, PosOps};

use super::event::{Event, Freq};

fn eval_event(e: &Cons) -> Vec<Box<Event>> {
    let mut ev = Vec::new();
    let mut pos = Pos { bar: 0, beat: 0, pos: 0.0 };
    match e {
        Cons::Cons(name, cdr) => {
            if let Cons::Cons(len, _) = &**cdr {
                ev.push(Box::new(Event::On(Pos {bar: 0, beat: 0, pos: 0.0}, 440.0)));
                ev.push(Box::new(Event::Off(Pos {bar: 0, beat: 0, pos: 0.25})));
            }
        },
        Cons::Symbol(name) => {
            match &name[..] {
                "loop" => ev.push(Box::new(Event::Loop(pos))),
                _ => panic!("unknown keyword or not implemented: {:?}", pos),
            }
        },
        sexp => {
            panic!("{:?} is not valid event", lisp::print(sexp));
        },
    }
    ev
}

fn eval_events(head: &Cons, rest: &Cons) -> Vec<Box<Event>> {
    let mut ev: Vec<Box<Event>> = Vec::new();
    ev.append(&mut eval_event(head));
    match rest {
        Cons::Cons(h, r) => ev.append(&mut eval_events(h, r)),
        _ => (),
    }
    ev
}

pub fn eval_one(sexp: &Cons) -> Vec<Box<Event>> {
    match sexp {
        Cons::Cons(car, cdr) => eval_events(car, cdr),
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
