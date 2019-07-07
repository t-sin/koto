use std::collections::HashMap;
use std::sync::Arc;

use super::super::units::unit::{Walk, UDump, Dump, Unit, AUnit};
use super::types::Env;

pub fn dump_unit(dump: &UDump) -> String {
    match dump {
        UDump::Str(s) => s.to_string(),
        UDump::Op(name, vec) => {
            let mut s = String::new();
            s.push_str("(");
            s.push_str(&name[..]);
            s.push_str(" ");
            for (i, d) in vec.iter().enumerate() {
                let dump = dump_unit(&**d);
                s.push_str(&dump[..]);
                if dump.len() != 0 && i != vec.len() - 1 {
                    s.push_str(" ");
                }
            }
            s.push_str(")");
            s
        }
    }
}

pub fn dump(ug: AUnit, env: &Env) -> String {
    let mut searched_units: Vec<AUnit> = Vec::new();
    let mut shared_units: Vec<AUnit> = Vec::new();
    let mut shared_unit_map = HashMap::new();
    let mut shared_id = 0;

    (*ug.0.lock().unwrap()).walk(&mut |u: &AUnit| {
        match searched_units.iter().position(|e| Arc::ptr_eq(e, u)) {
            Some(idx) => {
                let u = searched_units[idx].clone();
                match shared_units.iter().position(|e| Arc::ptr_eq(e, &u)) {
                    Some(_idx) => (),
                    None => {
                        shared_unit_map.insert(shared_units.len(), format!("$shared{}", shared_id));
                        shared_id += 1;
                        shared_units.push(u);
                    },
                }
                false
            },
            None => {
                searched_units.push(u.clone());
                true
            },
        }
    });

    let mut tlisp_str = String::new();
    tlisp_str.push_str(";; environment\n");
    let bpm_str = format!("(bpm {})\n", env.time.bpm);
    tlisp_str.push_str(&bpm_str.to_string());
    let mes_str = format!("(measure {} {})\n", env.time.measure.beat, env.time.measure.note);
    tlisp_str.push_str(&mes_str.to_string());

    tlisp_str.push_str("\n;; shared units\n");
    for (idx, su) in shared_units.iter().enumerate() {
        let dumped = dump_unit(&su.0.lock().unwrap().dump(&shared_units, &shared_unit_map));
        let name = shared_unit_map.get(&idx).unwrap().to_string();
        tlisp_str.push_str(&format!("(def {} {})\n", name, dumped));
    }

    tlisp_str.push_str("\n;; unit graph\n");
    let dumped = dump_unit(&ug.0.lock().unwrap().dump(&shared_units, &shared_unit_map));
    tlisp_str.push_str(&format!("{}\n", dumped));
    format!("{}", tlisp_str)
}
