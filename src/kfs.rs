use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use libc::{EACCES, ENOENT};
use time::Timespec;
use users::{get_current_gid, get_current_uid};

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyWrite, Request,
};

use super::mtime::Time;
use super::sexp::read;
use super::tapirlisp::types::{Env, EvalError};
use super::tapirlisp::{eval, TYPE_NAMES};
use super::ugen::core::{Aug, Dump, Operate, UgNode, Value};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

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

pub struct KotoFS {
    pub root: Arc<Mutex<KotoNode>>,
    pub inodes: HashMap<u64, Arc<Mutex<KotoNode>>>,
    pub augs: HashMap<Aug, Arc<Mutex<KotoNode>>>,
    pub sample_rate: u32,
    pub inode_count: u64,
}

fn create_file(ino: u64, size: u64, ftype: FileType) -> FileAttr {
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

impl KotoNode {
    fn create_node(ino: u64, name: String, data: Vec<u8>, ftype: FileType) -> KotoNode {
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

    fn parse_nodename(name: String) -> Option<(String, String)> {
        let nodename: Vec<&str> = name.split('.').collect();
        if nodename.len() == 2 {
            let paramname = nodename[0].to_string();
            let typename = nodename[1].to_string();
            Some((paramname, typename))
        } else {
            None
        }
    }

    fn get_nodename(node: Arc<Mutex<KotoNode>>) -> Option<(String, String)> {
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

    fn get_path_to_root(node: Arc<Mutex<KotoNode>>) -> String {
        if let Some(parent) = &node.lock().unwrap().parent {
            let mut parent_path = "../".to_string();
            parent_path.push_str(&KotoNode::get_path_to_root(parent.clone()));
            parent_path
        } else {
            "".to_string()
        }
    }

    fn get_path_from_root(node: Arc<Mutex<KotoNode>>) -> String {
        if let Some(parent) = &node.lock().unwrap().parent {
            let mut parent_path = KotoNode::get_path_from_root(parent.clone());
            if let Some((name, _)) = parent
                .lock()
                .unwrap()
                .children
                .iter()
                .find(|(_, n)| Arc::ptr_eq(&n, &node))
            {
                parent_path.push_str("");
                parent_path.to_string()
            } else {
                "".to_string()
            }
        } else {
            format!("/{}", node.lock().unwrap().name)
        }
    }

    fn build_ug_from_node(node: Arc<Mutex<KotoNode>>, sample_rate: u32) -> Option<Aug> {
        let name = node.lock().unwrap().name.clone();
        if let Ugen::Mapped(aug) = &node.lock().unwrap().ug {
            Some(aug.clone())
        } else {
            let form_str = "(aaa)".to_string();
            let mut env = Env::init(Time::new(sample_rate));
            match read(form_str.clone()) {
                Ok(form) => match eval(&form[0], &mut env) {
                    Ok(crate::tapirlisp::types::Value::Unit(aug)) => Some(aug.clone()),
                    Ok(crate::tapirlisp::types::Value::Nil) => {
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
    }

    fn sync_file(node: Arc<Mutex<KotoNode>>, oldname: String) {
        let data = node.lock().unwrap().data.clone();
        let data: String = if let Ok(data) = String::from_utf8(data.clone()) {
            data.clone()
        } else {
            println!("invalid data: {:?}", data.clone());
            return;
        };

        if let Some((paramname, _)) = KotoNode::get_nodename(node.clone()) {
            if let Some(parent) = &node.lock().unwrap().parent {
                if let Ugen::Mapped(ref mut aug) = &mut parent.lock().unwrap().ug {
                    if let Err(err) = aug.set_str(&paramname, data.clone()) {
                        println!("Error while setting '{}'", data.clone());
                        println!("{:?}", err);
                    }
                }
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

    fn sync_ug_with_directory(node: Arc<Mutex<KotoNode>>, oldname: String, sample_rate: u32) {
        if let Some((paramname, typename)) = KotoNode::get_nodename(node.clone()) {
            // nodename satisfies xxx.yyy format
            let set: HashSet<&str> = TYPE_NAMES.iter().cloned().collect();
            if set.contains(&typename[..]) {
                // typename (yyy of xxx.yyy) is valid
                if let Some(new_ug) = KotoNode::build_ug_from_node(node.clone(), sample_rate) {
                    if let Some(parent) = &node.lock().unwrap().parent {
                        if let Ugen::Mapped(ref mut parent_ug) = &mut parent.lock().unwrap().ug {
                            let _ = parent_ug.set(&paramname, new_ug.clone());
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

    fn sync_ug(node: Arc<Mutex<KotoNode>>, oldname: String, sample_rate: u32) {
        let filetype = node.lock().unwrap().attr.kind;
        match filetype {
            FileType::RegularFile => KotoNode::sync_file(node.clone(), oldname),
            FileType::Directory => {
                KotoNode::sync_ug_with_directory(node.clone(), oldname, sample_rate)
            }
            _ => (),
        }
    }
}

impl KotoFS {
    fn inode(&mut self) -> u64 {
        let ino = self.inode_count;
        self.inode_count += 1;
        ino
    }

    fn build_node_from_value(
        &mut self,
        v: Value,
        ug: Aug,
        parent: Arc<Mutex<KotoNode>>,
        shared: &Vec<Aug>,
        shared_used: &mut Vec<bool>,
    ) -> Arc<Mutex<KotoNode>> {
        match v {
            Value::Number(n) => {
                let data = n.to_string().into_bytes();
                let node = Arc::new(Mutex::new(KotoNode {
                    ug: Ugen::Mapped(ug.clone()),
                    parent: Some(parent),
                    children: [].to_vec(),
                    name: "val".to_string(),
                    data: n.to_string().into_bytes(),
                    link: None,
                    attr: create_file(self.inode(), data.len() as u64, FileType::RegularFile),
                }));
                self.augs.insert(ug.clone(), node.clone());
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());
                node
            }
            Value::Table(vec) => {
                let mut tab = String::new();
                for val in &vec {
                    tab.push_str(&format!("{}", val));
                    tab.push_str(" ");
                }
                tab.push_str("\n");
                let len = tab.len() as u64;
                let node = Arc::new(Mutex::new(KotoNode {
                    ug: Ugen::Mapped(ug.clone()),
                    parent: Some(parent),
                    children: [].to_vec(),
                    name: "tab".to_string(),
                    data: tab.into_bytes(),
                    link: None,
                    attr: create_file(self.inode(), len, FileType::RegularFile),
                }));
                self.augs.insert(ug.clone(), node.clone());
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());
                node
            }
            Value::Pattern(vec) => {
                let mut pat = String::new();
                for note in &vec {
                    pat.push_str(&note);
                    pat.push_str(" ");
                }
                pat.push_str("\n");
                let len = pat.len() as u64;
                let node = Arc::new(Mutex::new(KotoNode {
                    ug: Ugen::Mapped(ug.clone()),
                    parent: Some(parent),
                    children: [].to_vec(),
                    name: "pat".to_string(),
                    data: pat.into_bytes(),
                    link: None,
                    attr: create_file(self.inode(), len, FileType::RegularFile),
                }));
                self.augs.insert(ug.clone(), node.clone());
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());
                node
            }
            Value::Ug(aug) => {
                let node = self.build_node(aug.clone(), Some(parent), shared, shared_used);
                self.augs.insert(ug.clone(), node.clone());
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());
                node
            }
            Value::Shared(_, aug) => {
                let idx = shared.iter().position(|saug| *saug == aug).unwrap();
                if shared_used[idx] == false {
                    shared_used[idx] = true;
                    let node = Arc::new(Mutex::new(KotoNode {
                        ug: Ugen::Mapped(aug.clone()),
                        parent: Some(parent),
                        children: [].to_vec(),
                        name: "shared".to_string(),
                        data: [].to_vec(),
                        link: None,
                        attr: create_file(self.inode(), 0, FileType::RegularFile),
                    }));
                    self.augs.insert(aug.clone(), node.clone());
                    self.inodes
                        .insert(node.lock().unwrap().attr.ino, node.clone());
                    node
                } else {
                    if let Some(node) = self.augs.get(&aug) {
                        node.clone()
                    } else {
                        panic!("problem about shared ugens... why not found in self.augs...?")
                    }
                }
            }
        }
    }

    fn build_node(
        &mut self,
        ug: Aug,
        parent: Option<Arc<Mutex<KotoNode>>>,
        shared: &Vec<Aug>,
        shared_used: &mut Vec<bool>,
    ) -> Arc<Mutex<KotoNode>> {
        let ug_node = ug.dump(shared);
        match ug_node {
            UgNode::Val(v) => {
                let node =
                    self.build_node_from_value(v, ug.clone(), parent.unwrap(), shared, shared_used);
                self.augs.insert(ug.clone(), node.clone());
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());
                node
            }
            UgNode::Ug(name, slots) => {
                let node = Arc::new(Mutex::new(KotoNode {
                    ug: Ugen::Mapped(ug.clone()),
                    parent: parent,
                    children: [].to_vec(),
                    name: name,
                    data: [].to_vec(),
                    link: None,
                    attr: create_file(self.inode(), 0, FileType::Directory),
                }));
                self.augs.insert(ug.clone(), node.clone());
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());

                for s in slots.iter() {
                    let child = self.build_node_from_value(
                        s.value.clone(),
                        s.ug.clone(),
                        node.clone(),
                        shared,
                        shared_used,
                    );
                    let newname =
                        format!("{}.{}", s.name.clone(), child.lock().unwrap().name.clone());
                    node.lock().unwrap().children.push((newname, child.clone()));
                }
                node
            }
            UgNode::UgRest(name, slots, basename, values) => {
                let node = Arc::new(Mutex::new(KotoNode {
                    ug: Ugen::Mapped(ug.clone()),
                    parent: parent,
                    children: [].to_vec(),
                    name: name,
                    data: [].to_vec(),
                    link: None,
                    attr: create_file(self.inode(), 0, FileType::Directory),
                }));

                for s in slots.iter() {
                    let child = self.build_node_from_value(
                        s.value.clone(),
                        s.ug.clone(),
                        node.clone(),
                        shared,
                        shared_used,
                    );
                    let typename = child.lock().unwrap().name.clone();
                    let nodename = format!("{}.{}", s.name.clone(), typename);
                    node.lock()
                        .unwrap()
                        .children
                        .push((nodename, child.clone()));
                }
                for (i, v) in values.iter().enumerate() {
                    let child = self.build_node_from_value(
                        *v.clone(),
                        ug.clone(),
                        node.clone(),
                        shared,
                        shared_used,
                    );
                    let typename = child.lock().unwrap().name.clone();
                    child.lock().unwrap().name = format!("{}{}", basename, i);
                    let nodename = format!(
                        "{}.{}",
                        child.lock().unwrap().name.clone(),
                        typename.clone()
                    );
                    node.lock()
                        .unwrap()
                        .children
                        .push((nodename, child.clone()));
                }
                node
            }
        }
    }

    pub fn init(sample_rate: u32, ug: Aug) -> KotoFS {
        let mut fs = KotoFS {
            inodes: HashMap::new(),
            augs: HashMap::new(),
            root: Arc::new(Mutex::new(KotoNode {
                ug: Ugen::NotMapped,
                parent: None,
                children: Vec::new(),
                name: "".to_string(),
                data: "".to_string().into_bytes(),
                link: None,
                attr: create_file(0, 0, FileType::RegularFile),
            })),
            sample_rate: sample_rate,
            inode_count: 151,
        };

        let shared_ug = crate::ugen::util::collect_shared_ugs(ug.clone());
        let mut shared_used: Vec<bool> = shared_ug.iter().map(|_| false).collect();

        let root = fs.build_node(ug, None, &shared_ug, &mut shared_used);
        fs.root = root.clone();
        fs.root.lock().unwrap().attr.ino = 1;
        fs.inodes.insert(1, fs.root.clone());
        fs
    }

    pub fn mount(self, mountpoint: OsString) {
        fuse::mount(self, &mountpoint, &[]).expect(&format!("fail mount() with {:?}", mountpoint));
    }
}

impl Filesystem for KotoFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr() with {:?}", ino);

        if let Some(node) = self.inodes.get(&ino) {
            reply.attr(&TTL, &node.lock().unwrap().attr);
            return;
        } else {
            reply.error(ENOENT);
            return;
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir() from {:?}", ino);
        if offset > 0 {
            reply.ok();
            return;
        }

        if let Some(parent) = self.inodes.get(&ino) {
            reply.add(ino, 0, FileType::Directory, ".");
            reply.add(
                parent.lock().unwrap().attr.ino,
                1,
                FileType::Directory,
                "..",
            );
            let mut reply_add_offset = 2;

            for (name, node) in parent.lock().unwrap().children.iter() {
                let attr = node.lock().unwrap().attr;
                reply.add(attr.ino, reply_add_offset, attr.kind, name);
                reply_add_offset += 1;
            }
        }
        reply.ok();
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup() by {:?}", name);

        if let Some(parent_node) = self.inodes.get(&parent) {
            let children = &mut parent_node.lock().unwrap().children;
            let name = name.to_str().unwrap().to_string();

            if let Some((_, node)) = children.iter().find(|(nodename, _)| nodename == &name) {
                let attr = node.lock().unwrap().attr;
                reply.entry(&TTL, &attr, 0);
                return;
            }
        }

        reply.error(ENOENT);
    }

    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _flag: u32,
        reply: ReplyCreate,
    ) {
        println!("create() with {:?}", name);
        let ino = self.inode();
        let mut created: Option<Arc<Mutex<KotoNode>>> = None;
        let name = name.to_str().unwrap().to_string();

        if let Some(parent_node) = self.inodes.get(&parent) {
            let node = KotoNode::create_node(ino, name.clone(), [].to_vec(), FileType::RegularFile);
            let node = Arc::new(Mutex::new(node));
            node.lock().unwrap().parent = Some(parent_node.clone());
            parent_node
                .lock()
                .unwrap()
                .children
                .push((name.clone(), node.clone()));
            created = Some(node.clone());
        }

        if let Some(node) = created.clone() {
            self.inodes
                .insert(node.clone().lock().unwrap().attr.ino, node.clone());
        }

        if let Some(node) = created.clone() {
            KotoNode::sync_file(node.clone(), "".to_string());
            reply.created(&TTL, &node.lock().unwrap().attr, 0, 0, 0);
        }
    }

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<Timespec>,
        _mtime: Option<Timespec>,
        _fd: Option<u64>,
        _crtime: Option<Timespec>,
        _chgtime: Option<Timespec>,
        _bkuptime: Option<Timespec>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        println!("setattr() with {:?}", ino);
        match self.inodes.get(&ino) {
            Some(n) => reply.attr(&TTL, &n.lock().unwrap().attr),
            None => reply.error(EACCES),
        }
    }

    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, reply: ReplyEntry) {
        println!("mkdir() with {:?}", name);
        let ino = self.inode();
        if let Some(parent_node) = self.inodes.get(&parent) {
            let name = name.to_str().unwrap().to_string();
            let mut node =
                KotoNode::create_node(ino, name.clone(), [].to_vec(), FileType::Directory);
            node.parent = Some(parent_node.clone());

            let node = Arc::new(Mutex::new(node));
            parent_node
                .lock()
                .unwrap()
                .children
                .push((name, node.clone()));
            self.inodes
                .insert(node.lock().unwrap().attr.ino, node.clone());
            reply.entry(&TTL, &node.lock().unwrap().attr, 0);
            return;
        }
        reply.error(ENOENT);
    }

    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name = name.to_str().unwrap().to_string();
        let mut inode = None;

        if let Some(parent_node) = self.inodes.get(&parent) {
            let pos = parent_node
                .lock()
                .unwrap()
                .children
                .iter()
                .position(|(nodename, _)| nodename == &name);

            if let Some(idx) = pos {
                let (_, node) = &mut parent_node.lock().unwrap().children.remove(idx);
                inode = Some(node.lock().unwrap().attr.ino);
            }

            if let Ugen::Mapped(ref mut aug) = &mut parent_node.lock().unwrap().ug {
                if let Some((paramname, _)) = KotoNode::parse_nodename(name) {
                    aug.clear(&paramname);
                }
            }
        }

        if let Some(ino) = inode {
            self.inodes.remove(&ino);
        }

        reply.ok();
    }

    fn rename(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        _newparent: u64,
        newname: &OsStr,
        reply: ReplyEmpty,
    ) {
        println!("rename() {:?} to {:?}", name, newname);
        let reply_ok = true;
        let old_name = name.to_str().unwrap().to_string();
        let new_name = newname.to_str().unwrap().to_string();

        if let Some(parent_node) = self.inodes.get(&parent) {
            let children = &mut parent_node.lock().unwrap().children;
            if let Some(n) = children
                .iter()
                .position(|(nodename, _)| nodename == &old_name)
            {
                children[n].0 = new_name.clone();
            }
        }

        let node: Option<Arc<Mutex<KotoNode>>> = if let Some(parent_node) = self.inodes.get(&parent)
        {
            let children = &mut parent_node.lock().unwrap().children;
            if let Some(pos) = children
                .iter()
                .position(|(nodename, _)| nodename == &new_name)
            {
                Some(children[pos].1.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(node) = node {
            KotoNode::sync_ug(node.clone(), old_name.clone(), self.sample_rate);
        }

        if reply_ok {
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        println!("write() to {:?} with offset {:?}", ino, offset);
        let length: usize = data.len();
        if let Some(n) = self.inodes.get_mut(&ino) {
            if offset == 0 {
                n.lock().unwrap().attr.size = data.len() as u64;
                n.lock().unwrap().data = data.to_vec();
            } else {
                n.lock().unwrap().attr.size += data.len() as u64;
                n.lock().unwrap().data.append(&mut data.to_vec());
            }
        }

        if let Some(n) = self.inodes.get(&ino) {
            KotoNode::sync_file(n.clone(), "".to_string());
        }
        reply.written(length as u32);
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        println!("read() from {:?}", ino);
        match self.inodes.get(&ino) {
            Some(n) => {
                let data_rest = &n.lock().unwrap().data[offset as usize..];
                if data_rest.len() >= size as usize {
                    reply.data(&data_rest[..size as usize]);
                } else {
                    reply.data(&data_rest);
                }
            }
            None => reply.error(ENOENT),
        }
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        println!("unlink() {:?}", name);
        let mut inode = None;

        if let Some(parent_node) = self.inodes.get(&parent) {
            let name = name.to_str().unwrap().to_string();

            let pos = parent_node
                .lock()
                .unwrap()
                .children
                .iter()
                .position(|(nodename, _)| nodename == &name);
            if let Some(pos) = pos {
                let (_, node) = parent_node.lock().unwrap().children.remove(pos);
                inode = Some(node.lock().unwrap().attr.ino);
            }

            if let Ugen::Mapped(ref mut aug) = &mut parent_node.lock().unwrap().ug {
                if let Some((paramname, _)) = KotoNode::parse_nodename(name) {
                    aug.clear(&paramname);
                }
            }
        }

        if let Some(inode) = inode {
            self.inodes.remove(&inode);
            reply.ok();
            return;
        }

        reply.error(ENOENT);
    }

    fn readlink(&mut self, _req: &Request, ino: u64, reply: ReplyData) {
        println!("readlink() from {:?}", ino);

        if let Some(node) = self.inodes.get(&ino) {
            let mut is_link = false;
            match node.lock().unwrap().attr.kind {
                FileType::Symlink => is_link = true,
                _ => (),
            }

            if is_link == true {
                if let Some(ref path) = &node.lock().unwrap().link {
                    reply.data(path.as_path().to_str().unwrap().as_bytes());
                    return;
                }
            }
        }
        reply.error(ENOENT);
    }

    fn symlink(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        link: &Path,
        reply: ReplyEntry,
    ) {
        println!("symlink() with {:?}", name);
        let ino = self.inode();

        if let Some(parent_node) = self.inodes.get(&parent) {
            let name = name.to_str().unwrap().to_string();
            let path = Path::new(link.to_str().unwrap()).to_path_buf();
            let mut node = KotoNode::create_node(ino, name.clone(), [].to_vec(), FileType::Symlink);
            node.parent = Some(parent_node.clone());
            node.link = Some(path);

            let node = Arc::new(Mutex::new(node));
            parent_node
                .lock()
                .unwrap()
                .children
                .push((name.clone(), node.clone()));
            self.inodes
                .insert(node.lock().unwrap().attr.ino, node.clone());
            reply.entry(&TTL, &node.lock().unwrap().attr, 0);
            return;
        }
        reply.error(EACCES);
    }
}
