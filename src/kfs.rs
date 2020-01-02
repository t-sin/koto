use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::sync::{Arc, Mutex};

use libc::{EACCES, ENOENT};
use time::Timespec;
use users::{get_current_gid, get_current_uid};

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyWrite, Request,
};

use super::ugen::core::{Aug, Dump, Setv, UgNode, Value};

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
    pub attr: FileAttr,
}

pub struct KotoFS {
    pub root: Arc<Mutex<KotoNode>>,
    pub inodes: HashMap<u64, Arc<Mutex<KotoNode>>>,
    pub augs: HashMap<Aug, Arc<Mutex<KotoNode>>>,
    pub inode_count: u64,
}

fn get_ext(name: &str) -> String {
    let mut ext = String::new();
    for c in name.chars() {
        if c == '.' {
            break;
        }
        ext.push(c);
    }
    ext
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
        attr: attr,
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
                println!("shared-aug: {:?}", Arc::into_raw(aug.0.clone()));
                let idx = shared.iter().position(|saug| *saug == aug).unwrap();
                if shared_used[idx] == false {
                    shared_used[idx] = true;
                    let node = Arc::new(Mutex::new(KotoNode {
                        ug: Ugen::Mapped(aug.clone()),
                        parent: Some(parent),
                        children: [].to_vec(),
                        name: "shared".to_string(),
                        data: [].to_vec(),
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
                    node.lock().unwrap().name = s.name.clone();
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
                    child.lock().unwrap().name = s.name.clone();
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

    fn parse_nodename(&mut self, name: String) -> Option<(String, String)> {
        let nodename: Vec<&str> = name.split('.').collect();

        if nodename.len() == 2 {
            let paramname = nodename[0].to_string();
            let typename = nodename[1].to_string();
            Some((paramname, typename))
        } else {
            None
        }
    }

    fn sync_ug(&self, node: Arc<Mutex<KotoNode>>) {
        let filetype = node.lock().unwrap().attr.kind;
        match filetype {
            FileType::RegularFile => {
                let name = node.lock().unwrap().name.clone();
                println!("sync Aug named as {:?}", name);
                let data: String =
                    if let Ok(data) = String::from_utf8(node.lock().unwrap().data.clone()) {
                        data.clone()
                    } else {
                        panic!("invalid data for node {:?}", name);
                    };

                if let Ugen::Mapped(ref mut aug) = &mut node.lock().unwrap().ug {
                    let shared_ug = crate::ugen::util::collect_shared_ugs(aug.clone());
                    aug.setv(&name, data.clone(), &shared_ug);
                    println!("set {:?} to {:?}", data, name);
                } else {
                    println!("ooo not mapped...");
                }
            }
            FileType::Directory => {}
            _ => (),
        }
    }

            if let Some(parent) = self.inodes.get(&parent) {
                if let Ugen::Mapped(aug) = &parent.lock().unwrap().ug {
                    let shared_ug = crate::ugen::util::collect_shared_ugs(aug.clone());
                    return match aug.dump(&shared_ug) {
                        UgNode::Val(v) => Ugen::NotMapped,
                        UgNode::Ug(name, slots) => {
                            if let Some(slot) = slots.iter().find(|s| s.name == paramname) {
                                let uname = crate::ugen::util::get_ug_name(&slot.ug, &shared_ug);
                                if uname != typename {
                                    Ugen::NotMapped
                                } else {
                                    Ugen::Mapped(slot.ug.clone())
                                }
                            } else {
                                Ugen::NotMapped
                            }
                        }
                        UgNode::UgRest(_, slots, basename, values) => {
                            if let Some(slot) = slots.iter().find(|s| s.name == paramname) {
                                let uname = crate::ugen::util::get_ug_name(&slot.ug, &shared_ug);
                                if uname != typename {
                                    Ugen::NotMapped
                                } else {
                                    Ugen::Mapped(slot.ug.clone())
                                }
                            } else {
                                if let Ok(n) = paramname[basename.len()..].parse::<u64>() {
                                    match &*values[n as usize] {
                                        Value::Ug(aug) => Ugen::Mapped(aug.clone()),
                                        _ => Ugen::NotMapped,
                                    }
                                } else {
                                    panic!("invalid name");
                                }
                            }
                        }
                    };
                }
            }
        }
        return Ugen::NotMapped;
    }

    pub fn init(ug: Aug) -> KotoFS {
        let mut fs = KotoFS {
            inodes: HashMap::new(),
            augs: HashMap::new(),
            root: Arc::new(Mutex::new(KotoNode {
                ug: Ugen::NotMapped,
                parent: None,
                children: Vec::new(),
                name: "".to_string(),
                data: "".to_string().into_bytes(),
                attr: create_file(0, 0, FileType::RegularFile),
            })),
            inode_count: 151,
        };

        let shared_ug = crate::ugen::util::collect_shared_ugs(ug.clone());
        let mut shared_used: Vec<bool> = shared_ug.iter().map(|_| false).collect();

        let root = fs.build_node(ug, None, &shared_ug, &mut shared_used);
        fs.root = root.clone();
        fs.root.lock().unwrap().attr.ino = 1;
        fs.inodes.insert(1, fs.root.clone());
        for (_ino, node) in fs.inodes.iter() {
            let ino = node.lock().unwrap().attr.ino;
            let name = node.lock().unwrap().name.clone();
            println!("({}: '{}')", ino, name);
        }
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
            let node = create_node(ino, name.clone(), [].to_vec(), FileType::RegularFile);
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
            let ugen = self.map_ug(name.clone(), parent);
            node.lock().unwrap().ug = ugen;
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
            let mut node = create_node(ino, name.clone(), [].to_vec(), FileType::Directory);
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
        let ext = get_ext(newname.to_str().unwrap());
        println!("ext: {:?}", ext);

        if let Some(parent_node) = self.inodes.get(&parent) {
            let children = &parent_node.lock().unwrap().children;
            let old_name = name.to_str().unwrap();
            if let Some((_, node)) = children.iter().find(|(nodename, _)| nodename == &old_name) {
                node.lock().unwrap().name = newname.to_str().unwrap().to_string();
                reply.ok();
                return;
            }
        }
        // TODO: check the node can map to aug
        reply.error(ENOENT);
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
        self.sync_ug(ino);
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

    fn link(
        &mut self,
        _req: &Request,
        ino: u64,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEntry,
    ) {
        println!("link() {:?}", newname);

        if let Some(node) = self.inodes.get(&ino) {
            if let Some(parent) = self.inodes.get(&newparent) {
                let attr = node.lock().unwrap().attr;
                parent
                    .lock()
                    .unwrap()
                    .children
                    .push((newname.to_str().unwrap().to_string(), node.clone()));
                reply.entry(&TTL, &attr, 0);
                return;
            }
        }
        // TODO: connect ugs
        reply.error(ENOENT);
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        println!("unlink() {:?}", name);
        let mut inode = None;

        if let Some(parent_node) = self.inodes.get(&parent) {
            let children = &mut parent_node.lock().unwrap().children;
            let name = name.to_str().unwrap().to_string();

            if let Some(pos) = children.iter().position(|(nodename, _)| nodename == &name) {
                let (_, node) = &children[pos];
                inode = Some(node.lock().unwrap().attr.ino);
                children.remove(pos);
            }
        }

        if let Some(inode) = inode {
            self.inodes.remove(&inode);
            reply.ok();
            return;
        }
        // TODO: remove if no nodes

        reply.error(ENOENT);
    }
}
