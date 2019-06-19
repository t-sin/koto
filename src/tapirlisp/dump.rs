use super::super::units::unit::{Dump, Unit, Osc, Eg, AUnit, Node, UnitGraph};
use super::super::units::core::{Pan, Clip, Offset, Gain, Add, Multiply};
use super::super::units::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};
use super::super::units::sequencer::{AdsrEg, Seq};

pub fn dump_one(dump: &Dump) -> String {
    match dump {
        Dump::Str(s) => s.to_string(),
        Dump::Op(name, vec) => {
            let mut s = String::new();
            s.push_str("(");
            s.push_str(&name[..]);
            s.push_str(" ");
            for (i, d) in vec.iter().enumerate() {
                s.push_str(&dump_one(&**d)[..]);
                if i != vec.len() - 1 {
                    s.push_str(" ");
                }
            }
            s.push_str(")");
            s
        }
    }
}

pub fn dump(ug: AUnit) -> String {
    format!("{}", dump_one(&ug.0.lock().unwrap().dump()))
}
