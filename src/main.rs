extern crate cpal;
extern crate fuse;
extern crate libc;
extern crate num;
extern crate rand;

mod audio_device;
mod mtime;
mod event;
mod units;
mod sexp;
mod tapirlisp;
mod somnia;
mod kfs;

use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use audio_device::AudioDevice;
use mtime::{Time, Clock};

use units::unit::{Unit, AUnit};

use tapirlisp as tlisp;
use tapirlisp::value::{Value, Env};

struct SoundSystem {
    time: Time,
    root_ug: AUnit,
}

impl SoundSystem {
    pub fn new(time: Time, ug: AUnit) -> SoundSystem {
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

fn main() {
    let sample_rate = 44100u32;
    let time = Time::new(sample_rate);
    let mut env = Env::init(time);

    let mut f = File::open("./configure.lisp").unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text);

    let unit_graph = match tlisp::eval_all(sexp::read(text).unwrap(), &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
        Err(err) => panic!("Error!!! {:?}", err),
    };
    println!("{}", tlisp::dump(unit_graph.clone(), &env));

    let mut lcd = SoundSystem::new(env.time, unit_graph.clone());

    std::thread::spawn(move || {
        let audio_device = AudioDevice::open(lcd.time.sample_rate);
        lcd.run(&audio_device);
    });

    let mut fs = kfs::KotoFS::init();
    fs.build(unit_graph.clone());
    fs.mount(OsString::from("koto.test"));

    // somnia::run_test();
}
