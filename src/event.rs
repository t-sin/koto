use super::mtime::{Measure, Pos};

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

#[derive(Debug, Clone)]
pub enum Pitch {
    Pitch(NoteNum, Octave),
    Rest,
}

#[derive(Debug, Clone)]
pub enum Message {
    Note(Pitch, Pos),
    Loop,
}

pub fn to_note(name: &str) -> Pitch {
    let octave = match name.chars().nth(name.len() - 1) {
        // default octave is 4
        Some(c) => {
            if c.is_digit(10) {
                c.to_digit(10).unwrap()
            } else {
                4
            }
        }
        _ => 4,
    };
    let pitch = match name.chars().nth(0) {
        Some('a') => Pitch::Pitch(0, octave + 1),
        Some('b') => Pitch::Pitch(2, octave + 1),
        Some('c') => Pitch::Pitch(3, octave),
        Some('d') => Pitch::Pitch(5, octave),
        Some('e') => Pitch::Pitch(7, octave),
        Some('f') => Pitch::Pitch(8, octave),
        Some('g') => Pitch::Pitch(10, octave),
        Some('r') => Pitch::Rest,
        _ => panic!("invalid note name: {:?}", name),
    };
    if let Pitch::Rest = pitch {
        pitch
    } else {
        match name.chars().nth(1) {
            Some('+') => {
                if let Pitch::Pitch(n, o) = pitch {
                    Pitch::Pitch(n + 1, o)
                } else {
                    pitch
                }
            }
            Some('-') => {
                if let Pitch::Pitch(n, o) = pitch {
                    Pitch::Pitch(n - 1, o)
                } else {
                    pitch
                }
            }
            _ => pitch,
        }
    }
}

pub fn to_str(pitch: &Pitch) -> String {
    let mut s = String::new();
    let oct_fn = |o: u32, s: &mut String| {
        if o < 8 {
            s.push_str(&o.to_string());
        } else {
            panic!("invalid octave: {:?}", o);
        };
    };
    match pitch {
        Pitch::Pitch(0, o) => {
            s.push_str("a");
            oct_fn(*o - 1, &mut s)
        }
        Pitch::Pitch(1, o) => {
            s.push_str("a+");
            oct_fn(*o - 1, &mut s)
        }
        Pitch::Pitch(2, o) => {
            s.push_str("b");
            oct_fn(*o - 1, &mut s)
        }
        Pitch::Pitch(3, o) => {
            s.push_str("c");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(4, o) => {
            s.push_str("c+");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(5, o) => {
            s.push_str("d");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(6, o) => {
            s.push_str("d+");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(7, o) => {
            s.push_str("e");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(8, o) => {
            s.push_str("f");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(9, o) => {
            s.push_str("f+");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(10, o) => {
            s.push_str("g");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(11, o) => {
            s.push_str("g+");
            oct_fn(*o, &mut s)
        }
        Pitch::Pitch(n, _) => panic!("invalid note number: {:?}", n),
        Pitch::Rest => s.push_str("r"),
    }
    s
}

pub fn to_freq(pitch: &Pitch) -> Freq {
    if let Pitch::Pitch(n, o) = pitch {
        440.0 * 2.0f64.powf(*n as f64 / 12.0 + (*o as f64) - 5.0)
    } else {
        440.0
    }
}

pub fn to_pos(len: u32) -> Pos {
    let pos = if len == 0 {
        0.125
    } else if len >= 3 {
        2.0f64.powf(len as f64 - 3.0)
    } else {
        (len as f64 / 4.0)
    };
    Pos {
        bar: 0,
        beat: 0,
        pos: pos,
    }
}

pub fn to_len(p: &Pos, measure: &Measure) -> String {
    let Pos { bar, beat, pos } = p;

    let bar_beat = bar * measure.beat;
    let beat_pos = ((bar_beat + beat) * measure.note) as f64 + pos;
    let len = (beat_pos / 0.125).log(4.0) * 2.0;
    len.to_string()
}
