use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use libc::{EACCES, ENOENT};
use time::Timespec;

use fuse::{
    FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyWrite, Request,
};

use tapirus::musical_time::time::Transport;
use tapirus::ugens::core::{Aug, Dump, Operate, UgNode, Value};

use crate::kotonode::{create_file, KotoNode, Ugen};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

pub struct KotoFS {
    pub root: Arc<Mutex<KotoNode>>,
    pub inodes: HashMap<u64, Arc<Mutex<KotoNode>>>,
    pub augs: HashMap<Aug, Arc<Mutex<KotoNode>>>,
    pub transport: Arc<Mutex<Transport>>,
    pub lock: Arc<Mutex<bool>>,
    pub inode_count: u64,
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
                    let node = self.build_node(aug.clone(), Some(parent), shared, shared_used);
                    self.augs.insert(aug.clone(), node.clone());
                    self.inodes
                        .insert(node.lock().unwrap().attr.ino, node.clone());
                    node
                } else {
                    let node = Arc::new(Mutex::new(KotoNode {
                        ug: Ugen::Mapped(aug.clone()),
                        parent: Some(parent),
                        children: [].to_vec(),
                        name: "shared".to_string(),
                        data: [].to_vec(),
                        link: None,
                        attr: create_file(self.inode(), 0, FileType::Symlink),
                    }));
                    self.inodes
                        .insert(node.lock().unwrap().attr.ino, node.clone());
                    node
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

    fn modify_symlink(&self, pathmap: &Vec<(Arc<Mutex<KotoNode>>, String)>) {
        for (node, path) in pathmap.iter() {
            let mut is_symlink = false;
            if let FileType::Symlink = node.lock().unwrap().attr.kind {
                is_symlink = true;
            }
            if is_symlink {
                let aug = if let Ugen::Mapped(aug) = &node.lock().unwrap().ug {
                    Some(aug.clone())
                } else {
                    None
                };
                if let Some(aug) = aug {
                    if let Some((_, target_path)) = pathmap
                        .iter()
                        .find(|(n, _)| Arc::ptr_eq(n, &self.augs.get(&aug).unwrap()))
                    {
                        let path: Vec<&str> = path.split('/').collect();
                        let mut to_root = String::new();
                        for _ in 0..(path.len() - 2) {
                            to_root.push_str("../");
                        }
                        let link_path = format!("{}{}", to_root, target_path.split_at(1).1);
                        node.lock().unwrap().link = Some(PathBuf::from(link_path));
                    }
                }
            }
        }
    }

    pub fn init(transport: Arc<Mutex<Transport>>, ug: Aug, lock: Arc<Mutex<bool>>) -> KotoFS {
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
            transport: transport,
            lock: lock,
            inode_count: 151,
        };

        let shared_ug = tapirus::ugens::util::collect_shared_ugs(ug.clone());
        let mut shared_used: Vec<bool> = shared_ug.iter().map(|_| false).collect();

        let root = fs.build_node(ug, None, &shared_ug, &mut shared_used);

        let mut pathmap = Vec::new();
        KotoNode::build_pathmap(root.clone(), "".to_string(), &mut pathmap);
        fs.modify_symlink(&pathmap);
        fs.augs.clear();

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
        if offset > 0 {
            reply.ok();
            return;
        }

        if let Some(dirnode) = self.inodes.get(&ino) {
            reply.add(ino, 0, FileType::Directory, ".");

            let mut parent_ino = 1;
            if let Some(ref parent) = &dirnode.lock().unwrap().parent {
                parent_ino = parent.lock().unwrap().attr.ino as i64
            }
            reply.add(
                dirnode.lock().unwrap().attr.ino,
                parent_ino,
                FileType::Directory,
                "..",
            );
            let mut reply_add_offset = 2;

            for (name, node) in dirnode.lock().unwrap().children.iter() {
                let attr = node.lock().unwrap().attr;
                reply.add(attr.ino, reply_add_offset, attr.kind, name);
                reply_add_offset += 1;
            }
        }
        reply.ok();
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
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
        match self.inodes.get(&ino) {
            Some(node) => {
                if let Ok(_) = self.lock.lock() {
                    KotoNode::sync_ug(node.clone(), "".to_string(), self.transport.clone());
                }
                reply.attr(&TTL, &node.lock().unwrap().attr);
            }
            None => reply.error(EACCES),
        }
    }

    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, reply: ReplyEntry) {
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
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEmpty,
    ) {
        let reply_ok = true;
        let old_name = name.to_str().unwrap().to_string();
        let new_name = newname.to_str().unwrap().to_string();

        if parent == newparent {
            if let Some(parent_node) = self.inodes.get(&parent) {
                let children = &mut parent_node.lock().unwrap().children;
                if let Some(n) = children
                    .iter()
                    .position(|(nodename, _)| nodename == &old_name)
                {
                    children[n].0 = new_name.clone();
                    children[n].1.lock().unwrap().name = new_name.clone();
                }
            }
        } else {
            let mut node = None;
            if let Some(parent_node) = self.inodes.get(&parent) {
                let children = KotoNode::get_children(parent_node.clone());
                let mut pos = None;
                if let Some(n) = children
                    .iter()
                    .position(|(nodename, _)| nodename == &old_name)
                {
                    pos = Some(n);
                    node = Some(children[n].1.clone());
                }
                if let Some(pos) = pos {
                    parent_node.lock().unwrap().children.remove(pos);
                    if let Ok(_) = self.lock.lock() {
                        if let Some((paramname, _)) = KotoNode::parse_nodename(new_name.clone()) {
                            if let Ugen::Mapped(ref mut aug) = &mut parent_node.lock().unwrap().ug {
                                aug.clear(&paramname);
                            }
                        }
                    }
                }
            }

            if let Some(node) = node {
                if let Some(new_parent) = self.inodes.get(&newparent) {
                    node.lock().unwrap().parent = Some(new_parent.clone());
                    new_parent
                        .lock()
                        .unwrap()
                        .children
                        .push((new_name.clone(), node.clone()));
                    if let Ok(_) = self.lock.lock() {
                        if let Some((paramname, _)) = KotoNode::get_nodename(new_parent.clone()) {
                            if let Ugen::Mapped(ref mut aug) = &mut new_parent.lock().unwrap().ug {
                                aug.clear(&paramname);
                            }
                        }
                    }
                }
            }
        }

        let node: Option<Arc<Mutex<KotoNode>>> =
            if let Some(parent_node) = self.inodes.get(&newparent) {
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
            if let Ok(_) = self.lock.lock() {
                KotoNode::sync_ug(node.clone(), old_name.clone(), self.transport.clone());
            }
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
            if let Ok(_) = self.lock.lock() {
                KotoNode::sync_file(n.clone(), "".to_string());
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

            if let Ok(_) = self.lock.lock() {
                if let Ugen::Mapped(ref mut aug) = &mut parent_node.lock().unwrap().ug {
                    if let Some((paramname, _)) = KotoNode::parse_nodename(name) {
                        aug.clear(&paramname);
                    }
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
            if let Ok(_) = self.lock.lock() {
                KotoNode::sync_ug(node.clone(), "".to_string(), self.transport.clone());
            }
            reply.entry(&TTL, &node.lock().unwrap().attr, 0);
            return;
        }
        reply.error(EACCES);
    }
}
