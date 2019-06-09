use std::cmp::Ordering;
use std::cmp::Ord;

pub struct Pos {
    pub bar: u64,
    pub beat: u64,
    pub pos: f64,
}

pub struct Measure {
    pub beat: u64,
    pub note: u64,
}

pub struct Time {
    pub sample_rate: u32,
    pub tick: u64,
    pub bpm: f64,
    pub measure: Measure,
    pub pos: Pos,
}

pub trait PosOps<T> {
    fn add(&self, other: T, time: &Time) -> Pos;
}

impl PosOps<Pos> for Pos {
    fn add(&self, other: Pos, time: &Time) -> Pos {
        let pos_diff = self.pos + other.pos;
        let beat_diff = self.beat + other.beat + pos_diff.trunc() as u64;

        let new_pos = pos_diff.fract();
        let new_beat = beat_diff / time.measure.note;
        let new_bar =  self.bar + other.bar + (beat_diff / time.measure.beat);

        Pos { bar: new_bar, beat: new_beat, pos: new_pos }
    }
}

impl PosOps<(u64, u64, f64)> for Pos {
    fn add(&self, other: (u64, u64, f64), time: &Time) -> Pos {
        let t = Pos { bar: other.0, beat: other.1, pos: other.2 };
        self.add(t, &time)
    }
}

impl PosOps<f64> for Pos {
    fn add(&self, other: f64, time :&Time) -> Pos {
        self.add((0, 0, other), &time)
    }
}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.bar == other.bar && self.beat == other.beat && self.pos == other.pos
    }
}

impl Eq for Pos {}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else if self.bar > other.bar {
            Ordering::Greater
        } else if self.bar == other.bar && self.beat > other.beat {
            Ordering::Greater
        } else if self.bar == other.bar && self.beat == other.beat && self.pos > other.pos {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl Time {
    pub fn new(sample_rate: u32, bpm: f64) -> Time {
        Time {
            sample_rate: sample_rate,
            tick: 0,
            bpm: bpm,
            measure: Measure { beat: 4, note: 4 },
            pos: Pos { bar: 0, beat: 0, pos: 0.0 },
        }
    }
}

pub trait Clock {
    fn inc(&mut self);
}

impl Clock for Time {
    fn inc(&mut self) {
        self.tick += 1;

        // update pos
        let beat_diff = self.bpm / 60.0 / self.sample_rate as f64;
        self.pos = self.pos.add(beat_diff, &self);
    }
}
