use std::collections::HashMap;
use std::sync::Arc;

use super::super::ugen::core::{Walk, Param, Dump, Aug};
use super::types::Env;

fn dump_table(name: &String, vec: &Vec<f64>) -> String {
    let mut s = String::new();
    s.push_str("(");
    s.push_str(&name[..]);
    s.push_str(" ");
    for (i, v) in vec.iter().enumerate() {
        s.push_str(&v.to_string());
        if i != vec.len() - 1 {
            s.push_str(" ");
        }
    }
    s.push_str(")");
    s
}

fn dump_list(name: &String, vec: &Vec<String>) -> String {
    let mut s = String::new();
    s.push_str("(");
    s.push_str(&name[..]);
    s.push_str(" ");
    for (i, v) in vec.iter().enumerate() {
        s.push_str(v);
        if i != vec.len() - 1 {
            s.push_str(" ");
        }
    }
    s.push_str(")");
    s
}

fn dump_op(name: &String, vvec: &Vec<Box<Param>>) -> String {
    let mut s = String::new();
    s.push_str("(");
    s.push_str(&name[..]);
    s.push_str(" ");
    for (i, d) in vvec.iter().enumerate() {
        let dump = dump_unit(&**d);
        s.push_str(&dump[..]);
        if dump.len() != 0 && i != vvec.len() - 1 {
            s.push_str(" ");
        }
    }
    s.push_str(")");
    s
}

pub fn dump_unit(dump: &Param) -> String {
    match dump {
        Param::Value(s) => s.to_string(),
        Param::Table(vals) => dump_table(&"table".to_string(), vals),
        Param::Pattern(pat) => dump_list(&"pat".to_string(), pat),
        // TODO: Param::UgRest(name, _, vals) => dump_op(&name, &vvec),
        Param::Ug(name, _, vvec) => dump_op(&name, vvec),
        _ => "".to_string(),
    }
}

pub fn dump(ug: Aug, env: &Env) -> String {
    let mut searched_units: Vec<Aug> = Vec::new();
    let mut shared_units: Vec<Aug> = Vec::new();

    ug.walk(&mut |u: &Aug| {
        match searched_units.iter().position(|e| *e == *u) {
            Some(idx) => {
                let u = searched_units[idx].clone();
                match shared_units.iter().position(|e| *e == u) {
                    Some(_idx) => (),
                    None => {
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
        let dumped = dump_unit(&su.0.lock().unwrap().dump(&shared_units));
        tlisp_str.push_str(&format!("(def {} {})\n", format!("shared-{}", idx), dumped));
    }

    tlisp_str.push_str("\n;; unit graph\n");
    let dumped = dump_unit(&ug.dump(&shared_units));
    tlisp_str.push_str(&format!("{}\n", dumped));
    format!("{}", tlisp_str)
}
