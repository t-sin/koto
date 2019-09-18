use std::collections::HashMap;
use std::ffi::OsString;
use libc::ENOENT;
use time::Timespec;

use fuse::{Filesystem, FileType, Request, FileAttr, ReplyAttr, ReplyDirectory};

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

#[derive(Debug)]
pub struct KotoFS {
    // <ino, (parent_ino, pathname, fileattr)>
    pub inode_table: HashMap<u64, (u64, String, FileAttr)>,
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
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        for (&inode, f) in self.inode_table.iter() {
            if ino == inode {
                reply.attr(&TTL, &f.2);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn readdir(&mut self, _req: &Request, _ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if offset == 0 {
            reply.add(1, 0, FileType::Directory, ".");
            reply.add(2, 1, FileType::Directory, "..");
        }
        reply.ok();
    }
}
