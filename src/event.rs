use super::time::{Pos};

pub type Freq = f64;

#[derive(Debug)]
pub enum Event {
    On(Pos, Note),
    Off(Pos),
    Loop(Pos),
}

impl Clone for Event {
    fn clone(&self) -> Self {
        match self {
            Event::On(pos, note) => Event::On(pos.clone(), note.clone()),
            Event::Off(pos) => Event::Off(pos.clone()),
            Event::Loop(pos) => Event::Loop(pos.clone()),
        }
    }
}

pub type NoteNum = u32;
pub type Octave = u32;

#[derive(Debug, Clone)]
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

pub fn to_str(note: &Note) -> String {
    let mut s = String::new();
    match note {
        Note::Note(0, _) => s.push_str("a"),
        Note::Note(1, _) => s.push_str("a+"),
        Note::Note(2, _) => s.push_str("b"),
        Note::Note(3, _) => s.push_str("c"),
        Note::Note(4, _) => s.push_str("c+"),
        Note::Note(5, _) => s.push_str("d"),
        Note::Note(6, _) => s.push_str("d+"),
        Note::Note(7, _) => s.push_str("e"),
        Note::Note(8, _) => s.push_str("f"),
        Note::Note(9, _) => s.push_str("f+"),
        Note::Note(10, _) => s.push_str("g"),
        Note::Note(11, _) => s.push_str("g+"),
        Note::Note(n, _) => panic!("invalid note number: {:?}", n),
        Note::Rest => s.push_str("r"),
    }
    match note {
        Note::Note(_, o) if *o >= 0 && *o < 8 => s.push_str(&o.to_string()),
        Note::Note(_, o) => panic!("invalid octave: {:?}", o),
        Note::Rest => (),
    }
    s
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

pub fn to_len(pos: &Pos) -> String {
    let Pos { bar: bar, beat: beat, pos: pos } = pos;
    "1".to_string()
}
