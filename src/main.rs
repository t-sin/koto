extern crate cpal;
extern crate fuse;
extern crate libc;
extern crate num;
extern crate rand;


mod audiodevice;
mod mtime;
mod event;
mod ugen;
//mod soundsystem;

use audiodevice::AudioDevice;
//use soundsystem::SoundSystem;
use mtime::Time;

fn main() {
    let sample_rate = 44100u32;
    let time = Time::new(sample_rate);
    //let mut env = Env::init(time);

    // let mut f = File::open("./configure.lisp").unwrap();
    // let mut text = String::new();
    // f.read_to_string(&mut text);

    // let unit_graph = match tlisp::eval_all(sexp::read(text).unwrap(), &mut env) {
    //     Ok(Value::Unit(ug)) => ug,
    //     Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
    //     Err(err) => panic!("Error!!! {:?}", err),
    // };
    // println!("{}", tlisp::dump(unit_graph.clone(), &env));

    let mut ad = AudioDevice::open(sample_rate);
    // let mut lcd = SoundSystem::new(env.time, unit_graph.clone());
    // // std::thread::spawn(move || {
    // //     lcd.run(&ad);
    // // });
    // lcd.run(&ad);
    ad.run(|mut buf| {});

    // let mut fs = kfs::KotoFS::init();
    // fs.build(unit_graph.clone());
    // fs.mount(OsString::from("koto.test"));

    // somnia::run_test();
}
