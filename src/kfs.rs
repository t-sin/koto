use std::collections::HashMap;
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

use super::ugen::core::{Aug, Dump, UgNode, Value};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

#[derive(Clone)]
pub enum UgenState {
    NotMapped,
    Mapped(Aug),
}

#[derive(Clone)]
pub struct KotoNode {
    // if parent is None, it'a a root.
    pub parent: Option<Arc<Mutex<KotoNode>>>,
    pub ug: UgenState,
    pub children: Vec<Arc<Mutex<KotoNode>>>,
    pub name: String,
    pub data: Vec<u8>,
    pub link: PathBuf,
    pub attr: FileAttr,
}

pub struct KotoFS {
    pub root: Arc<Mutex<KotoNode>>,
    pub inodes: HashMap<u64, Arc<Mutex<KotoNode>>>,
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
        children: Vec::new(),
        parent: None,
        link: Path::new("").to_path_buf(),
        ug: UgenState::NotMapped,
        name: name,
        data: data,
        attr: attr,
    }
}

impl KotoFS {
    pub fn init() -> KotoFS {
        let attr = create_file(1000, 0, FileType::Directory);
        let file = KotoNode {
            children: [].to_vec(),
            ug: UgenState::NotMapped,
            name: "/".to_string(),
            data: [].to_vec(),
            link: Path::new("").to_path_buf(),
            parent: None,
            attr: attr,
        };
        let dummy = Arc::new(Mutex::new(file));
        let inodes = HashMap::new();
        KotoFS {
            inodes: inodes,
            root: dummy,
            inode_count: 151,
        }
    }

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
    ) -> Arc<Mutex<KotoNode>> {
        match v {
            Value::Number(n) => {
                let data = n.to_string().into_bytes();
                let node = KotoNode {
                    parent: Some(parent),
                    children: [].to_vec(),
                    ug: UgenState::Mapped(ug.clone()),
                    data: n.to_string().into_bytes(),
                    name: "val".to_string(),
                    link: Path::new("").to_path_buf(),
                    attr: create_file(self.inode(), data.len() as u64, FileType::RegularFile),
                };
                Arc::new(Mutex::new(node))
            }
            Value::Table(vec) => {
                let mut tab = String::new();
                for val in &vec {
                    tab.push_str(&format!("{}", val));
                    tab.push_str(" ");
                }
                tab.push_str("\n");
                let len = tab.len() as u64;
                let node = KotoNode {
                    parent: Some(parent),
                    children: [].to_vec(),
                    ug: UgenState::Mapped(ug.clone()),
                    data: tab.into_bytes(),
                    name: "tab".to_string(),
                    link: Path::new("").to_path_buf(),
                    attr: create_file(self.inode(), len, FileType::RegularFile),
                };
                Arc::new(Mutex::new(node))
            }
            Value::Pattern(vec) => {
                let mut pat = String::new();
                for note in &vec {
                    pat.push_str(&note);
                    pat.push_str(" ");
                }
                pat.push_str("\n");
                let len = pat.len() as u64;
                let node = KotoNode {
                    parent: Some(parent),
                    children: [].to_vec(),
                    ug: UgenState::Mapped(ug.clone()),
                    data: pat.into_bytes(),
                    name: "pat".to_string(),
                    link: Path::new("").to_path_buf(),
                    attr: create_file(self.inode(), len, FileType::RegularFile),
                };
                Arc::new(Mutex::new(node))
            }
            Value::Ug(aug) => {
                // TODO: UgenState::Mapped(aug.dump(shared))
                self.build_node(aug.clone(), parent, shared)
            }
            Value::Shared(_, aug) => {
                let node = KotoNode {
                    parent: Some(parent),
                    children: [].to_vec(),
                    ug: UgenState::Mapped(aug.clone()),
                    data: [].to_vec(),
                    name: "shared".to_string(),
                    link: Path::new("/").to_path_buf(),
                    attr: create_file(self.inode(), 0, FileType::Symlink),
                };
                Arc::new(Mutex::new(node))
            }
        }
    }

    fn build_node(
        &mut self,
        ug: Aug,
        parent: Arc<Mutex<KotoNode>>,
        shared: &Vec<Aug>,
    ) -> Arc<Mutex<KotoNode>> {
        let ug_node = ug.dump(shared);
        match ug_node {
            UgNode::Val(v) => {
                let node = self.build_node_from_value(v, ug.clone(), parent, shared);
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());
                node
            }
            UgNode::Ug(name, slots) => {
                let node = Arc::new(Mutex::new(KotoNode {
                    parent: Some(parent),
                    children: [].to_vec(),
                    ug: UgenState::Mapped(ug.clone()),
                    data: [].to_vec(),
                    name: name,
                    link: Path::new("").to_path_buf(),
                    attr: create_file(self.inode(), 0, FileType::Directory),
                }));
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());

                for s in slots.iter() {
                    let child = self.build_node_from_value(
                        s.value.clone(),
                        ug.clone(),
                        node.clone(),
                        shared,
                    );
                    let name = child.lock().unwrap().name.clone();
                    child.lock().unwrap().name = format!("{}.{}", s.name.clone(), name);
                    node.lock().unwrap().children.push(child.clone());
                    self.inodes
                        .insert(child.lock().unwrap().attr.ino, child.clone());
                }
                node
            }
            UgNode::UgRest(name, slots, basename, values) => {
                let node = Arc::new(Mutex::new(KotoNode {
                    parent: Some(parent),
                    children: [].to_vec(),
                    ug: UgenState::Mapped(ug.clone()),
                    data: [].to_vec(),
                    name: name,
                    link: Path::new("").to_path_buf(),
                    attr: create_file(self.inode(), 0, FileType::Directory),
                }));
                self.inodes
                    .insert(node.lock().unwrap().attr.ino, node.clone());

                for s in slots.iter() {
                    let child = self.build_node_from_value(
                        s.value.clone(),
                        ug.clone(),
                        node.clone(),
                        shared,
                    );
                    let name = child.lock().unwrap().name.clone();
                    child.lock().unwrap().name = format!("{}.{}", s.name.clone(), name);
                    node.lock().unwrap().children.push(child.clone());
                    self.inodes
                        .insert(child.lock().unwrap().attr.ino, child.clone());
                }
                for (i, v) in values.iter().enumerate() {
                    let child =
                        self.build_node_from_value(*v.clone(), ug.clone(), node.clone(), shared);
                    let name = child.lock().unwrap().name.clone();
                    child.lock().unwrap().name = format!("{}{}.{}", basename, i, name);
                    node.lock().unwrap().children.push(child.clone());
                    self.inodes
                        .insert(child.lock().unwrap().attr.ino, child.clone());
                }
                node
            }
        }
    }

    pub fn build(&mut self, ug: Aug) {
        let shared_ug = crate::ugen::util::collect_shared_ugs(ug.clone());
        let node = self.build_node(ug, self.root.clone(), &shared_ug);

        let ino = node.lock().unwrap().attr.ino;
        if let Some(node) = self.inodes.remove(&ino) {
            node.lock().unwrap().attr.ino = 1;
            self.root = node.clone();
            self.inodes
                .insert(node.lock().unwrap().attr.ino, node.clone());
        }
    }

    pub fn sync_ug(&mut self, _ino: u64) {}

    pub fn mount(self, mountpoint: OsString) {
        fuse::mount(self, &mountpoint, &[]).expect(&format!("fail mount() with {:?}", mountpoint));
    }
}

impl Filesystem for KotoFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr() with {:?}", ino);

        if let Some(node) = self.inodes.get(&ino) {
            println!("name = {}", node.lock().unwrap().name);
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

            for n in parent.lock().unwrap().children.iter() {
                let attr = n.lock().unwrap().attr;
                let name = n.lock().unwrap().name.to_string();
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

            if let Some(node) = children.iter().find(|n| n.lock().unwrap().name == name) {
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

        if let Some(parent_node) = self.inodes.get(&parent) {
            let name = name.to_str().unwrap().to_string();
            let mut node = create_node(ino, name, [].to_vec(), FileType::RegularFile);
            node.parent = Some(parent_node.clone());

            let node = Arc::new(Mutex::new(node));
            parent_node.lock().unwrap().children.push(node.clone());
            self.inodes
                .insert(node.lock().unwrap().attr.ino, node.clone());
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
            let mut node = create_node(ino, name, [].to_vec(), FileType::Directory);
            node.parent = Some(parent_node.clone());

            let node = Arc::new(Mutex::new(node));
            parent_node.lock().unwrap().children.push(node.clone());
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
            if let Some(node) = children
                .iter()
                .find(|n| &n.lock().unwrap().name == old_name)
            {
                node.lock().unwrap().name = newname.to_str().unwrap().to_string();
                reply.ok();
                return;
            }
        }
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
            let children = &mut parent_node.lock().unwrap().children;
            let name = name.to_str().unwrap().to_string();

            if let Some(pos) = children.iter().position(|n| n.lock().unwrap().name == name) {
                let node = &children[pos];
                inode = Some(node.lock().unwrap().attr.ino);
                children.remove(pos);
            }
        }

        if let Some(inode) = inode {
            self.inodes.remove(&inode);
            reply.ok();
            return;
        }

        reply.error(ENOENT);
    }
}
