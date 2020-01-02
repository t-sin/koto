use super::core::{Aug, Dump, UgNode, Walk};

pub fn collect_shared_ugs(ug: Aug) -> Vec<Aug> {
    let mut searched_units: Vec<Aug> = Vec::new();
    let mut shared_units: Vec<Aug> = Vec::new();

    ug.walk(
        &mut |u: &Aug| match searched_units.iter().position(|e| *e == *u) {
            Some(idx) => {
                let u = searched_units[idx].clone();
                match shared_units.iter().position(|e| *e == u) {
                    Some(_idx) => (),
                    None => {
                        shared_units.push(u);
                    }
                }
                false
            }
            None => {
                searched_units.push(u.clone());
                true
            }
        },
    );

    shared_units
}

pub fn get_ug_name(ug: &Aug, shared_ug: &Vec<Aug>) -> String {
    match ug.dump(&shared_ug) {
        UgNode::Ug(name, _) => name,
        UgNode::UgRest(name, _, _, _) => name,
        UgNode::Val(_) => "val".to_string(),
    }
}
