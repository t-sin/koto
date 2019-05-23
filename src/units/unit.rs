use super::super::time::Time;

pub trait Unit {
    fn calc(&self, time: &Time) -> f64;
    fn update(&mut self, time: &Time);
}
