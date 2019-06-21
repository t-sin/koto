use std::sync::Arc;

use super::super::units::unit::{Walk, Dump, Unit, Osc, Eg, AUnit, Node, UnitGraph};
use super::super::units::core::{Pan, Clip, Offset, Gain, Add, Multiply};
use super::super::units::oscillator::{Rand, Sine, Tri, Saw, Pulse, Phase, WaveTable};
use super::super::units::sequencer::{AdsrEg, Seq};

pub fn dump_unit(dump: &Dump) -> String {
    match dump {
        Dump::Str(s) => s.to_string(),
        Dump::Op(name, vec) => {
            let mut s = String::new();
            s.push_str("(");
            s.push_str(&name[..]);
            s.push_str(" ");
            for (i, d) in vec.iter().enumerate() {
                s.push_str(&dump_unit(&**d)[..]);
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
    let mut searched_units: Vec<AUnit> = Vec::new();
    let mut shared_units: Vec<AUnit> = Vec::new();
    (*ug.0.lock().unwrap()).walk(&mut |u: &AUnit| {
        match searched_units.iter_mut().find(|e| Arc::ptr_eq(e, &u)) {
            Some(u) => {
                shared_units.push(u.clone());
                false
            },
            None => {
                searched_units.push(u.clone());
                true
            },
        }
    });
    let mut tlisp_str = String::new();
    // TODO: dump env
    tlisp_str.push_str(";; environment\n");
    // dump shared units
    tlisp_str.push_str("\n;; shared units\n");
    for su in shared_units.iter() {
        // TODO: definition name
        tlisp_str.push_str(&format!("(def $hoge {})\n", dump_unit(&su.0.lock().unwrap().dump())));
    }
    // TOOD: dump unit graph
    tlisp_str.push_str("\n;; unit graph\n");
    tlisp_str.push_str(&format!("{}\n", dump_unit(&ug.0.lock().unwrap().dump())));
    format!("{}", tlisp_str)
}
