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
