use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use libc::{ENOENT, EACCES};
use time::Timespec;

use fuse::{
    Filesystem, FileType, Request, FileAttr,
    ReplyAttr, ReplyDirectory, ReplyEntry, ReplyCreate,
    ReplyWrite, ReplyData, ReplyEmpty
};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

#[derive(Debug, Clone)]
pub struct KotoNode {
    // if parent is None, it'a a root.
    pub parent: Option<Arc<Mutex<KotoNode>>>,
    pub inode: u64,
    pub children: Vec<Arc<Mutex<KotoNode>>>,
    pub name: String,
    pub data: Vec<u8>,
    pub link: PathBuf,
    pub attr: FileAttr,
}

#[derive(Debug)]
pub struct KotoFS {
    pub root: Arc<Mutex<KotoNode>>,
    pub inodes: HashMap<u64, Arc<Mutex<KotoNode>>>,
}

fn create_file(ino: u64, size: u64, ftype: FileType) -> FileAttr {
    let t = time::now().to_timespec();
    FileAttr {
        ino: ino, size: size, blocks: 0,
        atime: t, mtime: t, ctime: t, crtime: t,
        kind: ftype,
        perm: match ftype {
            FileType::Directory => 0o775,
            _ => 0o644,
        },
        nlink: 2, uid: 501, gid: 20, rdev: 0, flags: 0,
    }
}

impl KotoFS {
    pub fn init() -> KotoFS {
        let root = KotoNode {
            inode: 1, children: [].to_vec(),
            name: "/".to_string(), data: [].to_vec(), link: Path::new("").to_path_buf(),
            parent: None, attr: create_file(1, 0, FileType::Directory),
        };
        let root_arc = Arc::new(Mutex::new(root));
        let mut inodes = HashMap::new();
        inodes.insert(root_arc.lock().unwrap().inode, root_arc.clone());
        KotoFS { inodes: inodes, root: root_arc }
    }

    pub fn mount(self, mountpoint: OsString) {
        println!("{:?}", self);
        fuse::mount(self, &mountpoint, &[]).expect(&format!("fail mount() with {:?}", mountpoint));
    }
}

impl Filesystem for KotoFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr() with {:?}", ino);
        if let None = self.inodes.get(&ino) {
            reply.error(ENOENT);
            return;
        }

        if let Some(node) = self.inodes.get(&ino) {
            reply.attr(&TTL, &node.lock().unwrap().attr);
            return;
        }

        reply.error(ENOENT);
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        println!("readdir() from {:?}", ino);
        if offset > 0 {
            reply.ok();
            return;
        }
        reply.add(1, 0, FileType::Directory, ".");
        reply.add(2, 1, FileType::Directory, "..");
        let mut reply_add_offset = 2;

        if let Some(parent) = self.inodes.get(&ino) {
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

    fn create(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, _flag: u32, reply: ReplyCreate) {
        println!("create() with {:?}", name);
        if let Some(parent_node) = self.inodes.get(&parent) {
            // TODO: check if exist a same filename

            let inode = time::now().to_timespec().sec as u64;
            let f = create_file(inode, 0, FileType::RegularFile);
            let node = KotoNode {
                parent: Some(parent_node.clone()), inode: inode, children: Vec::new(),
                name: name.to_str().unwrap().to_string(), data: [].to_vec(), link: Path::new("").to_path_buf(), attr: f,
            };
            let node = Arc::new(Mutex::new(node));
            parent_node.lock().unwrap().children.push(node.clone());
            self.inodes.insert(inode, node);
            reply.created(&TTL, &f, 0, 0, 0,);
        }
    }

    fn setattr(&mut self, _req: &Request, ino: u64, _mode: Option<u32>, _uid: Option<u32>, _gid: Option<u32>,
        _size: Option<u64>, _atime: Option<Timespec>, _mtime: Option<Timespec>, _fd: Option<u64>,
        _crtime: Option<Timespec>, _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>,
        reply: ReplyAttr) {
        println!("setattr() with {:?}", ino);
        match self.inodes.get(&ino) {
            Some(n) => reply.attr(&TTL, &n.lock().unwrap().attr),
            None => reply.error(EACCES),
        }
    }

    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, reply: ReplyEntry) {
        println!("mkdir() with {:?}", name);
        if let Some(parent_node) = self.inodes.get(&parent) {
            let inode = time::now().to_timespec().sec as u64;
            let attr = create_file(inode, 0, FileType::Directory);
            let node = KotoNode {
                parent: Some(parent_node.clone()), inode: inode, children: Vec::new(),
                name: name.to_str().unwrap().to_string(), data: [].to_vec(), link: Path::new("").to_path_buf(), attr: attr,
            };

            let node = Arc::new(Mutex::new(node));
            parent_node.lock().unwrap().children.push(node.clone());
            self.inodes.insert(inode, node.clone());
            reply.entry(&TTL, &node.lock().unwrap().attr, 0);
            return;
        }
        reply.error(ENOENT);
    }

    fn rename(&mut self, _req: &Request, parent: u64, name: &OsStr, _newparent: u64, newname: &OsStr, reply: ReplyEmpty) {
        println!("rename() {:?} to {:?}", name, newname);
        let mut ext: String = "".to_string();
        for c in newname.to_str().unwrap().to_string().chars() {
            if c == '.' {
                break;
            }
            ext.push(c);
        }
        println!("ext: {:?}", ext);

        if let Some(parent_node) = self.inodes.get(&parent) {
            let children = &parent_node.lock().unwrap().children;
            let old_name = name.to_str().unwrap();
            if let Some(node) = children.iter().find(|n| &n.lock().unwrap().name == old_name) {
                node.lock().unwrap().name = newname.to_str().unwrap().to_string();
                reply.ok();
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn write(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, data: &[u8], _flags: u32, reply: ReplyWrite) {
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

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        println!("read() from {:?}", ino);
        match self.inodes.get(&ino) {
            Some(n) => {
                let data_rest = &n.lock().unwrap().data[offset as usize..];
                if data_rest.len() >= size as usize {
                    reply.data(&data_rest[..size as usize]);
                } else {
                    reply.data(&data_rest);
                }
            },
            None => reply.error(EACCES),
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
                inode = Some(node.lock().unwrap().inode);
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

    fn readlink(&mut self, _req: &Request, ino: u64, reply: ReplyData) {
        println!("readlink() from {:?}", ino);

        if let Some(node) = self.inodes.get(&ino) {
            let mut is_link = false;
            match node.lock().unwrap().attr.kind {
                FileType::Symlink => is_link = true,
                _ => (),
            }

            if is_link == true {
                let path = &node.lock().unwrap().link;
                reply.data(path.as_path().to_str().unwrap().as_bytes());
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn symlink(&mut self, _req: &Request, parent: u64, name: &OsStr, link: &Path, reply: ReplyEntry) {
        println!("symlink() with {:?}", name);

        if let Some(parent_node) = self.inodes.get(&parent) {
            // TODO: check if exist a same filename

            let inode = time::now().to_timespec().sec as u64;
            let attr = create_file(inode, 0, FileType::Symlink);
            let path = Path::new(link.to_str().unwrap()).to_path_buf();
            let node = KotoNode {
                parent: Some(parent_node.clone()), inode: inode, children: Vec::new(),
                name: name.to_str().unwrap().to_string(), data: [].to_vec(), link: path, attr: attr,
            };

            let node = Arc::new(Mutex::new(node));
            parent_node.lock().unwrap().children.push(node.clone());
            self.inodes.insert(inode, node);
            reply.entry(&TTL, &attr, 0);
            return;
        }
        reply.error(EACCES);
    }
}
