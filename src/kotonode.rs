use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fuse::{FileAttr, FileType};
use users::{get_current_gid, get_current_uid};

use tapirus::musical_time::time::Transport;
use tapirus::tapirlisp::eval::{eval, TYPE_NAMES};
use tapirus::tapirlisp::sexp::read;
use tapirus::tapirlisp::types::{Env, Value};
use tapirus::ugens::core::{Aug, Dump, Operate, UgNode};

#[derive(Clone)]
pub enum Ugen {
    NotMapped,
    Mapped(Aug),
}

#[derive(Clone)]
pub struct KotoNode {
    pub ug: Ugen,
    pub parent: Option<Arc<Mutex<KotoNode>>>,
    pub children: Vec<(String, Arc<Mutex<KotoNode>>)>,
    pub name: String,
    pub data: Vec<u8>,
    pub link: Option<PathBuf>,
    pub attr: FileAttr,
}

impl KotoNode {
    pub fn create_node(ino: u64, name: String, data: Vec<u8>, ftype: FileType) -> KotoNode {
        let size = match ftype {
            FileType::RegularFile => data.len(),
            _ => 0,
        };
        let attr = create_file(ino, size as u64, ftype);
        KotoNode {
            ug: Ugen::NotMapped,
            parent: None,
            children: Vec::new(),
            name: name,
            data: data,
            link: None,
            attr: attr,
        }
    }

    pub fn parse_nodename(name: String) -> Option<(String, String)> {
        let nodename: Vec<&str> = name.split('.').collect();
        if nodename.len() == 2 {
            let paramname = nodename[0].to_string();
            let typename = nodename[1].to_string();
            Some((paramname, typename))
        } else {
            None
        }
    }

    pub fn get_nodename(node: Arc<Mutex<KotoNode>>) -> Option<(String, String)> {
        if let Some(parent) = &node.lock().unwrap().parent {
            if let Some((nodename, _)) = parent
                .lock()
                .unwrap()
                .children
                .iter()
                .find(|(_, n)| Arc::ptr_eq(&n, &node))
            {
                KotoNode::parse_nodename(nodename.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_children(node: Arc<Mutex<KotoNode>>) -> Vec<(String, Arc<Mutex<KotoNode>>)> {
        let mut children = Vec::new();
        for (name, child) in node.lock().unwrap().children.iter() {
            children.push((name.clone(), child.clone()));
        }
        children
    }

    fn resolve_symlink_1(
        path: &[&str],
        node: Arc<Mutex<KotoNode>>,
    ) -> Option<Arc<Mutex<KotoNode>>> {
        if path.len() == 0 {
            Some(node.clone())
        } else {
            match path[0] {
                ".." => {
                    let mut parent = None;
                    if let Some(parent_node) = &node.lock().unwrap().parent {
                        parent = Some(parent_node.clone())
                    }
                    if let Some(parent) = parent {
                        KotoNode::resolve_symlink_1(&path[1..], parent.clone())
                    } else {
                        None
                    }
                }
                name => {
                    if let Some((_, next)) = KotoNode::get_children(node.clone())
                        .iter()
                        .find(|(n, _)| name == n)
                    {
                        KotoNode::resolve_symlink_1(&path[1..], next.clone())
                    } else {
                        None
                    }
                }
            }
        }
    }

    pub fn resolve_symlink(node: Arc<Mutex<KotoNode>>) -> Option<Arc<Mutex<KotoNode>>> {
        let mut link = node
            .lock()
            .unwrap()
            .link
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if link.chars().nth(link.len() - 1).unwrap() == '/' {
            link = link[..link.len() - 1].to_string();
        }
        let path: Vec<&str> = link.split('/').collect();
        let parent = &node.lock().unwrap().parent;
        if let Some(parent) = parent {
            KotoNode::resolve_symlink_1(&path, parent.clone())
        } else {
            None
        }
    }

    pub fn build_pathmap(
        node: Arc<Mutex<KotoNode>>,
        path: String,
        pathmap: &mut Vec<(Arc<Mutex<KotoNode>>, String)>,
    ) {
        for (name, child) in node.lock().unwrap().children.iter() {
            let child_path = format!("{}/{}", path, name);
            pathmap.push((child.clone(), child_path.clone()));
            KotoNode::build_pathmap(child.clone(), child_path.clone(), pathmap);
        }
    }

    pub fn build_ug_from_node(
        node: Arc<Mutex<KotoNode>>,
        transport: Arc<Mutex<Transport>>,
    ) -> Option<Aug> {
        let name = node.lock().unwrap().name.clone();
        if let Ugen::Mapped(aug) = &node.lock().unwrap().ug {
            return Some(aug.clone());
        }

        println!("building {}.", name.clone());
        let (_, name) = KotoNode::parse_nodename(name.clone()).unwrap();
        let mut env = Env::init(Transport::new(transport.lock().unwrap().sample_rate));
        let form_str = match &name[..] {
            "pan" => "(pan 0 0)",
            "clip" => "(clip 0 0 0)",
            "offset" => "(offset 0 0)",
            "gain" => "(gain 0 0)",
            "+" => "(+)",
            "*" => "(*)",
            "oneshot" => {
                println!("I'm not a cat!");
                "(oneshot 0 0)"
            }
            "rand" => "(rand 0)",
            "sine" => "(sine 0 0)",
            "tri" => "(tri 0 0)",
            "saw" => "(saw 0 0)",
            "pulse" => "(pulse 0 0 0)",
            "table" => "(table 0 0)",
            "phase" => "(phase 0)",
            "wavetable" => "(wavetable (table -1 -1 -1 1 1 1) 0)",
            "pat" => "(pat)",
            "trig" => "(trig 0 0)",
            "adsr" => "(adsr 0 0 0 0)",
            "seq" => "(seq 0 0 0 0)",
            "lpf" => "(lpf 0 0 0)",
            "delay" => "(delay 0 0 0 0)",
            "out" => "(out 0 0)",
            _ => "0",
        };

        match read(form_str.to_string()) {
            Ok(form) => match eval(&form[0], &mut env) {
                Ok(Value::Unit(mut aug)) => {
                    node.lock().unwrap().ug = Ugen::Mapped(aug.clone());
                    let dump = aug.dump(&vec![]);
                    println!("sexp = {}", form_str);
                    match dump {
                        UgNode::Val(_val) => (),
                        UgNode::Ug(_, _slots) => {
                            let mut children = Vec::new();
                            for (name, child) in node.lock().unwrap().children.iter() {
                                children.push((name.clone(), child.clone()));
                            }

                            for (name, child) in children.iter() {
                                if let Some((paramname, _)) = KotoNode::parse_nodename(name.clone())
                                {
                                    KotoNode::sync_ug(
                                        child.clone(),
                                        "".to_string(),
                                        transport.clone(),
                                    );
                                    if let Ugen::Mapped(child_ug) = &child.lock().unwrap().ug {
                                        let _ = aug.set(&paramname, child_ug.clone());
                                    }
                                }
                            }
                        }
                        UgNode::UgRest(_, _slots, paramname, _values) => {
                            let mut children = Vec::new();
                            for (name, child) in node.lock().unwrap().children.iter() {
                                children.push((name.clone(), child.clone()));
                            }

                            for (name, child) in children.iter() {
                                if let Some((paramname, _)) = KotoNode::parse_nodename(name.clone())
                                {
                                    KotoNode::sync_ug(
                                        child.clone(),
                                        "".to_string(),
                                        transport.clone(),
                                    );
                                    if let Ugen::Mapped(child_ug) = &child.lock().unwrap().ug {
                                        let _ = aug.set(&paramname, child_ug.clone());
                                    }
                                }
                            }

                            for (name, child) in children.iter() {
                                if let Some((child_paramname, _)) =
                                    KotoNode::parse_nodename(name.clone())
                                {
                                    if child_paramname.starts_with(&paramname) {
                                        KotoNode::sync_ug(
                                            child.clone(),
                                            "".to_string(),
                                            transport.clone(),
                                        );
                                        if let Ugen::Mapped(child_ug) = &child.lock().unwrap().ug {
                                            let _ = aug.set(&paramname, child_ug.clone());
                                        }
                                    }
                                }
                            }
                        }
                    };
                    Some(aug.clone())
                }
                Ok(Value::Nil) => {
                    println!("'build_ug_from_node' is wrong");
                    None
                }
                Err(err) => {
                    println!("cannot evaluate this node: {:?}", form_str);
                    println!("{:?}", err);
                    None
                }
            },
            Err(err) => {
                println!("cannot parse this node: {:?}", name);
                println!("{:?}", err);
                None
            }
        }
    }

    pub fn sync_file(node: Arc<Mutex<KotoNode>>, oldname: String) {
        let data = node.lock().unwrap().data.clone();
        let data: String = if let Ok(data) = String::from_utf8(data.clone()) {
            data.clone()
        } else {
            println!("invalid data: {:?}", data.clone());
            return;
        };

        if let Some((paramname, _)) = KotoNode::get_nodename(node.clone()) {
            let mut node_ug = None;
            if let Some(parent) = &node.lock().unwrap().parent {
                if let Ugen::Mapped(ref mut aug) = &mut parent.lock().unwrap().ug {
                    let mut data = data.clone();
                    data.retain(|c| c != '\n');
                    if let Err(err) = aug.set_str(&paramname, data.clone()) {
                        println!("Error while setting '{}'", data.clone());
                        println!("{:?}", err);
                    }
                    if let Ok(ug) = aug.get(&paramname) {
                        node_ug = Some(ug.clone());
                    }
                }
            }
            if let Some(ug) = node_ug {
                node.lock().unwrap().ug = Ugen::Mapped(ug.clone());
            }
        } else {
            if let Some((paramname, _)) = KotoNode::parse_nodename(oldname.clone()) {
                if let Some(parent) = &node.lock().unwrap().parent {
                    if let Ugen::Mapped(ref mut aug) = &mut parent.lock().unwrap().ug {
                        aug.clear(&paramname);
                    }
                }
            }
        }
    }

    pub fn sync_directory(
        node: Arc<Mutex<KotoNode>>,
        oldname: String,
        transport: Arc<Mutex<Transport>>,
    ) {
        if let Some((paramname, typename)) = KotoNode::get_nodename(node.clone()) {
            // nodename satisfies xxx.yyy format
            let set: HashSet<&str> = TYPE_NAMES.iter().cloned().collect();
            if set.contains(&typename[..]) {
                // typename (yyy of xxx.yyy) is valid
                if let Some(new_ug) = KotoNode::build_ug_from_node(node.clone(), transport.clone())
                {
                    if let Some(parent) = &node.lock().unwrap().parent {
                        if let Ugen::Mapped(ref mut parent_ug) = &mut parent.lock().unwrap().ug {
                            let _ = parent_ug.set(&paramname, new_ug.clone());
                        }
                    }
                }
            } else if &typename[..] == "shared" {
                println!("aaaaaaaaaaaaa");
                let mut aug = None;
                if let Ugen::Mapped(ug) = &node.lock().unwrap().ug {
                    aug = Some(ug.clone());
                }
                if let Some(ug) = &aug {
                    if let Some(parent) = &node.lock().unwrap().parent {
                        if let Ugen::Mapped(ref mut parent_ug) = &mut parent.lock().unwrap().ug {
                            let _ = parent_ug.set(&paramname, ug.clone());
                        }
                    }
                }
            } else {
                // paramname (xxx of xxx.yyy) is changed (or filename is not changed)
                if let Some(parent) = &node.lock().unwrap().parent {
                    if let Ugen::Mapped(ref mut parent_ug) = &mut parent.lock().unwrap().ug {
                        {
                            let _ = parent_ug.clear(&paramname);
                        }
                    }
                }
            }
        } else {
            // nodename not satisfies xxx.yyy format
            if let Some((paramname, _)) = KotoNode::parse_nodename(oldname.clone()) {
                if let Some(parent) = &node.lock().unwrap().parent {
                    if let Ugen::Mapped(ref mut aug) = &mut parent.lock().unwrap().ug {
                        aug.clear(&paramname);
                    }
                }
            }
        }
    }

    pub fn sync_symlink(node: Arc<Mutex<KotoNode>>) {
        let target = KotoNode::resolve_symlink(node.clone());
        if let Some((paramname, _)) = KotoNode::get_nodename(node.clone()) {
            if let Some(target) = target {
                let mut target_aug = None;
                if let Ugen::Mapped(aug) = &target.lock().unwrap().ug {
                    target_aug = Some(aug.clone());
                }

                if let Some(aug) = target_aug {
                    let mut parent = None;
                    if let Some(parent_node) = &node.lock().unwrap().parent {
                        parent = Some(parent_node.clone());
                    }

                    if let Some(parent) = parent {
                        if let Ugen::Mapped(ref mut parent_aug) = &mut parent.lock().unwrap().ug {
                            // this Aug.set() causes deadlock but why...???
                            let _ = parent_aug.set(&paramname, aug.clone());
                            node.lock().unwrap().ug = Ugen::Mapped(aug.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn sync_ug(node: Arc<Mutex<KotoNode>>, oldname: String, transport: Arc<Mutex<Transport>>) {
        let filetype = node.lock().unwrap().attr.kind;
        match filetype {
            FileType::RegularFile => KotoNode::sync_file(node.clone(), oldname),
            FileType::Directory => {
                KotoNode::sync_directory(node.clone(), oldname, transport.clone())
            }
            FileType::Symlink => KotoNode::sync_symlink(node.clone()),
            _ => (),
        }
    }
}

pub fn create_file(ino: u64, size: u64, ftype: FileType) -> FileAttr {
    let t = time::now().to_timespec();
    FileAttr {
        ino: ino,
        size: size,
        blocks: 0,
        atime: t,
        mtime: t,
        ctime: t,
        crtime: t,
        kind: ftype,
        perm: match ftype {
            FileType::Directory => 0o775,
            _ => 0o644,
        },
        uid: get_current_uid(),
        gid: get_current_gid(),
        nlink: 2,
        rdev: 0,
        flags: 0,
    }
}
