use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use libc::{ENOENT, EACCES};
use time::Timespec;

use fuse::{
    Filesystem, FileType, Request, FileAttr,
    ReplyAttr, ReplyDirectory, ReplyEntry, ReplyCreate, ReplyWrite, ReplyData,
};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

#[derive(Debug)]
pub struct KotoFS {
    // <ino, (parent_ino, pathname, fileattr)>
    pub inode_table: HashMap<u64, (u64, String, FileAttr)>,
    pub data_table: HashMap<u64, Vec<u8>>,
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
        let mut kfs = KotoFS {
            inode_table: HashMap::new(),
            data_table: HashMap::new(),
        };
        kfs.inode_table.insert(1, (0, "/".to_string(), create_file(1, 0, FileType::Directory)));
        kfs
    }

    pub fn mount(self, mountpoint: OsString) {
        println!("{:?}", self);
        fuse::mount(self, &mountpoint, &[]).expect(&format!("fail mount() with {:?}", mountpoint));
    }
}

impl Filesystem for KotoFS {
    // get attribute for the entry identified by inode `ino`
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr() with {:?}", ino);
        for (&inode, f) in self.inode_table.iter() {
            if ino == inode {
                reply.attr(&TTL, &f.2);
                return;
            }
        }
        reply.error(ENOENT);
    }

    // get directory entries in `ino`
    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if offset > 0 {
            reply.ok();
            return;
        }
        reply.add(1, 0, FileType::Directory, ".");
        reply.add(2, 1, FileType::Directory, "..");
        let mut reply_add_offset = 2;
        for (_, f) in self.inode_table.iter() {
            if ino == f.0 {
                let attr = f.2;
                let name = &f.1;
                reply.add(attr.ino, reply_add_offset, attr.kind, name);
                reply_add_offset += 1;
            }
        }
        reply.ok();
    }

    // lookup() checks if the entry `name` exists
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup() by {:?}", name);
        for (_, f) in self.inode_table.iter() {
            if f.0 == parent && name.to_str().unwrap() == f.1.as_str() {
                reply.entry(&TTL, &f.2, 0);
                return;
            }
        }
        reply.error(ENOENT);
    }

    // create file as `name`
    fn create(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, _flag: u32, reply: ReplyCreate) {
        println!("create() with {:?}", name);
        let inode = time::now().to_timespec().sec as u64;
        let f = create_file(inode, 0, FileType::RegularFile);
        self.inode_table.insert(inode, (parent, name.to_str().unwrap().to_string(), f));
        reply.created(&TTL, &f, 0, 0, 0,);
    }

    // set attribute to `ino`
    fn setattr(&mut self, _req: &Request, ino: u64, _mode: Option<u32>, _uid: Option<u32>, _gid: Option<u32>,
        _size: Option<u64>, _atime: Option<Timespec>, _mtime: Option<Timespec>, _fd: Option<u64>,
        _crtime: Option<Timespec>, chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>,
        reply: ReplyAttr) {
        match self.inode_table.get(&ino) {
            Some(f) => reply.attr(&TTL, &f.2),
            None => reply.error(EACCES),
        }
    }

    fn write(&mut self, _req: &Request, ino: u64, _fh: u64, _offset: i64, data: &[u8], _flags: u32, reply: ReplyWrite) {
        let length: usize = data.len();
        let d = data.to_vec();
        self.data_table.insert(ino, d);
        if let Some(f) = self.inode_table.get_mut(&ino) {
            let parent_ino = f.0;
            let name = f.1.clone();
            *f = (parent_ino, name, create_file(ino, length as u64, FileType::RegularFile));
        }
        reply.written(length as u32);
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, _offset: i64, _size: u32, reply: ReplyData) {
        match self.data_table.get(&ino) {
            Some(d) => reply.data(d),
            None => reply.error(EACCES),
        }
    }
}
