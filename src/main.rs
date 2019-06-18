mod audio_device;
mod time;
mod event;
mod units;
mod tapirlisp;

use audio_device::AudioDevice;
use time::{Time, Clock};

use units::unit::Unit;

use tapirlisp::types::{Value, Env};
use tapirlisp as tlisp;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let time = Time::new(sample_rate, 120.0);
    let mut env = Env::init(time);

    let s = r"
(def $pat1 (pat ((c 1) (r 1) (c 0) (r 0) (r 1) (d 2) (r 2)
                (e 1) (r 1) (e 0) (r 0) (r 1) (f 2) (r 2)
                (g 2) (r 2) (a 1) (r 1) (r 1) (r 0) (a 0)
                (b 2) (r 2) (c5 2) (r 2)
                loop)))
(def $osc1 (wavetable (saw 0 1) (phase (saw 0 440))))
(def $eg1 (adsr 0 (gain 0.2 (offset 1 (saw 0 0.25))) 0.0 0.1))

(def $pat2 (pat ((c 2) (r 2) (c 2) (r 2) (c 2) (r 2) (c 2) (r 2) loop)))
(def $osc2 (rand 0))
(def $eg2 (adsr 0 0.1 0.005 0))

(+ (gain 0.3 (seq $pat1 $osc1 $eg1))
   (gain 0.25 (seq $pat2 $osc2 $eg2)))
".to_string();
    let unit_graph = match tlisp::eval_all(tlisp::read(s).unwrap(), &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
        Err(err) => panic!("Error!!! {:?}", err),
    };

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.lock().unwrap().calc(&env.time).0 as f32;
            unit_graph.lock().unwrap().update(&env.time);
            env.time.inc();
        }
    });
}
