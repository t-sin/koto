use super::super::tapirlisp as lisp;
use super::super::tapirlisp::Cons;

use super::super::time::{Pos, Measure, PosOps, Time};

use super::event::{Event, Freq};

enum Note {
    R, C(u32), Cs(u32), D(u32), Ds(u32), E(u32), F(u32),
    Fs(u32), G(u32), Gs(u32), A(u32), As(u32), B(u32),
}

fn to_note(name: &str) -> Note {
    let octave = match name.chars().nth(name.len() - 1) {
        // default octave is 4
        Some(c) => if c.is_digit(10) { c.to_digit(10).unwrap() } else { 4 }
        _ => 4,
    };
    match name.chars().nth(0) {
        Some('c') => match name.chars().nth(1) {
            Some('+') => Note::Cs(octave),
            Some('-') => Note::B(octave - 1),
            _ => Note::C(octave),
        },
        Some('d') => match name.chars().nth(1) {
            Some('+') => Note::Ds(octave),
            Some('-') => Note::Cs(octave),
            _ => Note::D(octave),
        },
        Some('e') => match name.chars().nth(1) {
            Some('+') => Note::F(octave),
            Some('-') => Note::Ds(octave),
            _ => Note::E(octave),
        },
        Some('f') => match name.chars().nth(1) {
            Some('+') => Note::Fs(octave),
            Some('-') => Note::E(octave),
            _ => Note::F(octave),
        },
        Some('g') => match name.chars().nth(1) {
            Some('+') => Note::Gs(octave),
            Some('-') => Note::Fs(octave),
            _ => Note::G(octave),
        },
        Some('a') => match name.chars().nth(1) {
            Some('+') => Note::As(octave),
            Some('-') => Note::Gs(octave),
            _ => Note::A(octave),
        },
        Some('b') => match name.chars().nth(1) {
            Some('+') => Note::C(octave + 1),
            Some('-') => Note::As(octave),
            _ => Note::B(octave),
        },
        Some('r') => Note::R,
        _ => Note::R,
    }
}

fn freq(note_num: u32, octave: u32) -> f64 {
    440.0 + (440.0 * note_num as f64 / 12.0) / (4.0 / octave as f64)
}

fn to_freq(n: Note) -> Freq {
    match n {
        Note::A(o) => freq(0, o),
        Note::As(o) => freq(1, o),
        Note::B(o) => freq(2, o),
        Note::C(o) => freq(3, o),
        Note::Cs(o) => freq(4, o),
        Note::D(o) => freq(5, o),
        Note::Ds(o) => freq(6, o),
        Note::E(o) => freq(7, o),
        Note::F(o) => freq(8, o),
        Note::Fs(o) => freq(9, o),
        Note::G(o) => freq(10, o),
        Note::Gs(o) => freq(11, o),
        Note::R => 442.0,  // whatever
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
    let mut time = Time { // TODO: read from global settings
        sample_rate: 0, tick: 0, bpm: 0.0,  // not used
        pos: Pos { bar: 0, beat: 0, pos: 0.0 },  // not used
        measure: Measure { beat: 4, note: 4 }
    };

    match e {
        Cons::Cons(name, cdr) => {
            if let Cons::Symbol(n) = &**name {
                if let Cons::Cons(len, _) = &**cdr {
                    let len = match &**len {
                        Cons::Symbol(l) => if l.chars().fold(true, |a, c| a && c.is_digit(10)) {
                            to_pos(l.parse::<u32>().unwrap())
                        } else {
                            to_pos(4)
                        },
                        _ => to_pos(4),
                    };
                    match to_note(&n) {
                        Note::R => {
                            *pos = pos.add(len, &time);
                        },
                        n => {
                            ev.push(Box::new(Event::On(pos.clone(), to_freq(n))));
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
