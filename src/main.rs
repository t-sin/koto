extern crate clap;
extern crate fuse;
extern crate libc;
extern crate signal_hook;
extern crate users;

extern crate tapirus;

mod kotofs;
mod kotonode;

use std::ffi::OsString;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

use clap::{App, Arg};

use tapirus::audiodevice::AudioDevice;
use tapirus::musical_time::time::Transport;
use tapirus::soundsystem::SoundSystem;
use tapirus::tapirlisp as tlisp;
use tapirus::tapirlisp::types::{Env, Value};

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
        init_config = "(out 0.25 0)".to_string();
    }

    let mountpoint = matches
        .value_of("mountpoint")
        .unwrap_or("koto.test")
        .to_string();

    let sample_rate = 44100u32;
    let transport = Transport::new(sample_rate);
    let mut env = Env::init(transport);

    let ug = match tlisp::eval_all(tlisp::sexp::read(init_config).unwrap(), &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
        Err(err) => panic!("Error!!! {:?}", err),
    };

    let lock = Arc::new(Mutex::new(true));

    let ug_clone = ug.clone();
    let env_clone = env.clone();
    let lock_clone = lock.clone();
    let signal = unsafe {
        signal_hook::register(signal_hook::SIGUSR1, move || {
            let filename = format!(
                "koto.{}.lisp",
                time::strftime("%Y%m%dT%H%M%S", &time::now()).unwrap()
            );
            let mut f = File::create(filename).unwrap();
            let mut config = None;
            if let Ok(_) = lock_clone.lock() {
                config = Some(tlisp::dump(ug_clone.clone(), &env_clone));
            }
            if let Some(config) = config {
                let _ = f.write_all(config.as_bytes());
            }
        })
    }
    .unwrap();

    let transport = Arc::new(Mutex::new(env.transport));
    let ad = AudioDevice::open(sample_rate);
    let mut lcd = SoundSystem::new(transport.clone(), ug.clone(), lock.clone());
    std::thread::spawn(move || {
        lcd.run(&ad);
    });

    let fs = kotofs::KotoFS::init(transport.clone(), ug.clone(), lock.clone());
    fs.mount(OsString::from(mountpoint));

    // somnia::run_test();

    signal_hook::unregister(signal);
}
