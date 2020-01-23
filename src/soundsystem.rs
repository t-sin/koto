use std::sync::{Arc, Mutex};

use super::mtime::{Clock, Time};
use super::ugen::core::{Aug, Proc};

use super::audiodevice::AudioDevice;

pub struct SoundSystem {
    time: Arc<Mutex<Time>>,
    root_ug: Aug,
}

impl SoundSystem {
    pub fn new(time: Arc<Mutex<Time>>, ug: Aug) -> SoundSystem {
        SoundSystem {
            time: time,
            root_ug: ug,
        }
    }

    pub fn run(&mut self, ad: &AudioDevice) {
        ad.run(|mut buffer| {
            let mut iter = buffer.iter_mut();
            loop {
                let mut time = self.time.lock().unwrap();
                let (l, r) = self.root_ug.0.lock().unwrap().proc(&time);
                time.inc();

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
