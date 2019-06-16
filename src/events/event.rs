use super::super::time::{Pos};

pub type Freq = f64;

#[derive(Debug)]
pub enum Event {
    On(Pos, Freq),
    Off(Pos),
    Loop(Pos),
}

impl Clone for Event {
    fn clone(&self) -> Self {
        match self {
            Event::On(pos, freq) => Event::On(pos.clone(), *freq),
            Event::Off(pos) => Event::Off(pos.clone()),
            Event::Loop(pos) => Event::Loop(pos.clone()),
        }
    }
}

pub type NoteNum = u32;
pub type Octave = u32;

pub enum Note {
    Note(NoteNum, Octave),
    Rest,
}

pub fn to_note(name: &str) -> Note {
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

pub fn to_freq(note: &Note) -> Freq {
    if let Note::Note(n, o) = note {
        440.0 * 2.0f64.powf(*n as f64 / 12.0 + (*o as f64) - 5.0)
    } else {
        440.0
    }
}

pub fn to_pos(len: u32) -> Pos {
    let pos = if len == 0 {
        0.125
    } else if len > 4 {
        2.0f64.powf(len as f64 - 4.0)
    } else {
        (len as f64 / 4.0)
    };
    Pos { bar: 0, beat: 0, pos: pos }
}
