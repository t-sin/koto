use super::mtime::{Clock, Time};
use super::ugen::core::{Aug, Proc};

use super::audiodevice::AudioDevice;

pub struct SoundSystem {
    time: Time,
    root_ug: Aug,
}

impl SoundSystem {
    pub fn new(time: Time, ug: Aug) -> SoundSystem {
        SoundSystem {
            time: time,
            root_ug: ug,
        }
    }

    pub fn run(&mut self, ad: &AudioDevice) {
        ad.run(|mut buffer| {
            let mut iter = buffer.iter_mut();
            loop {
                let (l, r) = self.root_ug.0.lock().unwrap().proc(&self.time);
                match iter.next() {
                    Some(lref) => *lref = l as f32,
                    None => break,
                }
                match iter.next() {
                    Some(rref) => *rref = r as f32,
                    None => break,
                }
                self.time.inc();
            }
        });
    }
}
