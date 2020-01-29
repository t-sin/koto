extern crate cpal;
extern crate fuse;
extern crate libc;
extern crate num;
extern crate rand;
extern crate signal_hook;
extern crate users;

extern crate clap;

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

use clap::{App, Arg};

use audiodevice::AudioDevice;
use mtime::Time;
use soundsystem::SoundSystem;

use tapirlisp as tlisp;
use tapirlisp::types::{Env, Value};

fn main() {
    let matches = App::new("Koto - music performing filesystem")
        .version("0.9.0")
        .author("t-sin <shinichi.tanaka45@gmail.com>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG.LISP")
                .help("Sets initial synthesizer configuration"),
        )
        .arg(
            Arg::with_name("mountpoint")
                .help("Specifies mount point")
                .required(true),
        )
        .get_matches();

    let mut init_config: String;
    if let Some(config) = matches.value_of("config") {
        init_config = String::new();
        let mut f = File::open(config).unwrap();
        let _ = f.read_to_string(&mut init_config);
    } else {
        init_config = "(out 0 0)".to_string();
    }

    let mountpoint = matches
        .value_of("mountpoint")
        .unwrap_or("koto.test")
        .to_string();

    let sample_rate = 44100u32;
    let time = Time::new(sample_rate);
    let mut env = Env::init(time);

    let ug = match tlisp::eval_all(sexp::read(init_config).unwrap(), &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
        Err(err) => panic!("Error!!! {:?}", err),
    };

    let ug_clone = ug.clone();
    let env_clone = env.clone();
    let signal = unsafe {
        signal_hook::register(signal_hook::SIGUSR1, move || {
            let filename = format!(
                "koto.{}.lisp",
                time::strftime("%Y%m%dT%H%M%S", &time::now()).unwrap()
            );
            let mut f = File::create(filename).unwrap();
            let config = tlisp::dump(ug_clone.clone(), &env_clone);
            let _ = f.write_all(config.as_bytes());
        })
    }
    .unwrap();

    let lock = Arc::new(Mutex::new(true));
    let time = Arc::new(Mutex::new(env.time));
    let ad = AudioDevice::open(sample_rate);
    let mut lcd = SoundSystem::new(time.clone(), ug.clone(), lock.clone());
    std::thread::spawn(move || {
        lcd.run(&ad);
    });

    let fs = kfs::KotoFS::init(time.clone(), ug.clone(), lock.clone());
    fs.mount(OsString::from(mountpoint));

    // somnia::run_test();

    signal_hook::unregister(signal);
}
