use super::super::tapirlisp as lisp;
use super::super::tapirlisp::cons::Cons;

use super::super::time::{Pos, Measure, PosOps, Time};

use super::event::{Event, Freq};

type NoteNum = u32;
type Octave = u32;

enum Note {
    Note(NoteNum, Octave),
    Rest,
}

fn to_note(name: &str) -> Note {
    let octave = match name.chars().nth(name.len() - 1) {
        // default octave is 4
        Some(c) => if c.is_digit(10) { c.to_digit(10).unwrap() } else { 4 }
        _ => 4,
    };
    let note = match name.chars().nth(0) {
        Some('a') => Note::Note(0, octave + 1),
        Some('b') => Note::Note(2, octave + 1),
        Some('c') => Note::Note(3, octave),
        Some('d') => Note::Note(5, octave),
        Some('e') => Note::Note(7, octave),
        Some('f') => Note::Note(8, octave),
        Some('g') => Note::Note(10, octave),
        Some('r') => Note::Rest,
        _ => panic!("invalid note name: {:?}", name),
    };
    if let Note::Rest = note {
        note
    } else {
        match name.chars().nth(1) {
            Some('+') => {
                if let Note::Note(n, o) = note {
                    Note::Note(n + 1, o)
                } else {
                    note
                }
            },
            Some('-') => {
                if let Note::Note(n, o) = note {
                    Note::Note(n - 1, o)
                } else {
                    note
                }
            },
            _ => note,
        }
    }
}

fn to_freq(note: &Note) -> Freq {
    if let Note::Note(n, o) = note {
        440.0 * 2.0f64.powf(*n as f64 / 12.0 + (*o as f64) - 5.0)
    } else {
        440.0
    }
}

fn to_pos(len: u32) -> Pos {
    let pos = if len == 0 {
        0.125
    } else if len > 4 {
        2.0f64.powf(len as f64 - 4.0)
    } else {
        (len as f64 / 4.0)
    };
    Pos { bar: 0, beat: 0, pos: pos }
}

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
