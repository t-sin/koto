extern crate cpal;
extern crate fuse;
extern crate libc;
extern crate num;
extern crate rand;
extern crate users;

mod audiodevice;
mod event;
mod kfs;
mod mtime;
mod sexp;
mod soundsystem;
mod tapirlisp;
mod ugen;

use std::ffi::OsString;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

use audiodevice::AudioDevice;
use mtime::Time;
use soundsystem::SoundSystem;

use tapirlisp as tlisp;
use tapirlisp::types::{Env, Value};

fn main() {
    let sample_rate = 44100u32;
    let time = Time::new(sample_rate);
    let mut env = Env::init(time);

    let mut f = File::open("./configure.lisp").unwrap();
    let mut text = String::new();
    let _ = f.read_to_string(&mut text);

    let ug = match tlisp::eval_all(sexp::read(text).unwrap(), &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
        Err(err) => panic!("Error!!! {:?}", err),
    };
    println!("{}", tlisp::dump(ug.clone(), &env));

    let lock = Arc::new(Mutex::new(true));
    let time = Arc::new(Mutex::new(env.time));
    let ad = AudioDevice::open(sample_rate);
    let mut lcd = SoundSystem::new(time.clone(), ug.clone(), lock.clone());
    std::thread::spawn(move || {
        lcd.run(&ad);
    });

    let fs = kfs::KotoFS::init(time.clone(), ug.clone(), lock.clone());
    fs.mount(OsString::from("koto.test"));

    // somnia::run_test();
}
