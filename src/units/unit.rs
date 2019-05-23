use super::super::time::Time;

pub trait Unit<'a> {
    fn calc(&self, time: &Time) -> f64;
    fn update(&mut self, time: &Time);
}

pub enum UnitGraph<'a> {
    Value(f64),
    Unit(dyn Unit<'a>),
    Units(Vec<&'a dyn Unit<'a>>),
}

impl Unit<'a> for UnitGraph<'a> {
    fn calc(&self, time: &Time) -> f64 {
        match self {
            UnitGraph::Value(v) => *v,
            UnitGraph::Unit(u) => u.calc(&time),
            UnitGraph::Units(us) =>  us.iter().fold(0.0, |acc, s| acc + s.calc(&time)),
        }
    }

    fn update(&mut self, time: &Time) {
        match self {
            UnitGraph::Value(v) => (),
            UnitGraph::Unit(u) => u.update(&time),
            UnitGraph::Units(us) => us.iter_mut().for_each(move |s| s.update(&time)),
        }
    }
}
