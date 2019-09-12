use std::collections::HashMap;
use std::ffi::OsString;

use fuse::{Filesystem, FileAttr};

#[derive(Debug)]
pub struct KotoFS {
    pub inode_table: HashMap<u64, (u64, String, FileAttr)>,
}

impl Filesystem for KotoFS {}

impl KotoFS {
    pub fn init() -> KotoFS {
        KotoFS {
            inode_table: HashMap::new(),
        }
    }

    pub fn mount(self, mountpoint: OsString) {
        println!("{:?}", self);
        fuse::mount(self, &mountpoint, &[]).expect("fail mount()");
    }
}
