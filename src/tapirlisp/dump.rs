use super::super::ugen::core::{UgNode, Slot, Value, Dump, Aug};
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

pub fn dump_value(v: &Value, shared: &Vec<Aug>) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::Table(vals) => dump_table(&"table".to_string(), vals),
        Value::Pattern(pat) => dump_list(&"pat".to_string(), pat),
        Value::Ug(ug) => dump_unit(&ug.dump(shared), shared),
        Value::Shared(n, _aug) => format!("shared-{}", n),
    }
}

fn dump_ug(name: &String, slots: &Vec<Slot>, values: &Vec<Box<Value>>, shared: &Vec<Aug>) -> String {
    let mut s = String::new();
    s.push_str("(");
    s.push_str(&name[..]);
    s.push_str(" ");
    for (i, u) in slots.iter().enumerate() {
        let dump = dump_value(&u.value, shared);
        s.push_str(&dump[..]);
        if dump.len() != 0 && i != slots.len() - 1 || values.len() > 0 {
            s.push_str(" ");
        }
    }
    if values.len() > 0 {
        for (i, v) in values.iter().enumerate() {
            s.push_str(&dump_value(&v, shared)[..]);
            if i != values.len() - 1 {
                s.push_str(" ");
            }
        }
    }

    s.push_str(")");
    s
}

pub fn dump_unit(dump: &UgNode, shared: &Vec<Aug>) -> String {
    match dump {
        UgNode::Val(v) => dump_value(v, shared),
        UgNode::Ug(name, slots) => dump_ug(&name, slots, &Vec::new(), shared),
        UgNode::UgRest(name, slots, _, values) => dump_ug(&name, slots, values, shared),
    }
}


pub fn dump(ug: Aug, env: &Env) -> String {
    let mut shared_units = crate::ugen::util::collect_shared_ugs(ug.clone());

    let mut tlisp_str = String::new();
    tlisp_str.push_str(";; environment\n");
    let bpm_str = format!("(bpm {})\n", env.time.bpm);
    tlisp_str.push_str(&bpm_str.to_string());
    let mes_str = format!("(measure {} {})\n", env.time.measure.beat, env.time.measure.note);
    tlisp_str.push_str(&mes_str.to_string());

    tlisp_str.push_str("\n;; shared units\n");
    for (idx, su) in shared_units.iter().enumerate() {
        let dumped = dump_unit(&su.0.lock().unwrap().dump(&shared_units), &shared_units);
        tlisp_str.push_str(&format!("(def {} {})\n", format!("shared-{}", idx), dumped));
    }

    tlisp_str.push_str("\n;; unit graph\n");
    let dumped = dump_unit(&ug.dump(&shared_units), &shared_units);
    tlisp_str.push_str(&format!("{}\n", dumped));
    format!("{}", tlisp_str)
}
