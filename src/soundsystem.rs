use std::sync::{Arc, Mutex};

use super::mtime::{Clock, Time};
use super::ugen::core::{Aug, Proc};

use super::audiodevice::AudioDevice;

pub struct SoundSystem {
    time: Arc<Mutex<Time>>,
    root_ug: Aug,
    lock: Arc<Mutex<bool>>,
}

impl SoundSystem {
    pub fn new(time: Arc<Mutex<Time>>, ug: Aug, lock: Arc<Mutex<bool>>) -> SoundSystem {
        SoundSystem {
            time: time,
            root_ug: ug,
            lock: lock,
        }
    }

    pub fn run(&mut self, ad: &AudioDevice) {
        ad.run(|mut buffer| {
            let mut iter = buffer.iter_mut();
            loop {
                let (mut l, mut r) = (0.0, 0.0);
                if let Ok(_) = self.lock.lock() {
                    let mut time = self.time.lock().unwrap();
                    let s = self.root_ug.0.lock().unwrap().proc(&time);
                    l = s.0;
                    r = s.1;
                    time.inc();
                }

                match iter.next() {
                    Some(lref) => *lref = l as f32,
                    None => break,
                }
                match iter.next() {
                    Some(rref) => *rref = r as f32,
                    None => break,
                }
            }
        });
    }
}
