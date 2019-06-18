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

    let s = r"(seq (pat ((c 2) (r 2)   (d 2) (r 2)
                         (e 2) (r 2)   (f 2) (r 2)
                         (g 2) (r 2)   (a 2) (r 2)
                         (b 2) (r 2)   (c5 2) (r 2)
                         loop))
                   (wavetable (saw 0 1) (phase (saw 0 440)))
                   (adsr 0 (gain 0.2 (offset 1 (saw 0 0.25))) 0.0 0.1))".to_string();
    let unit_graph = match tlisp::eval(&tlisp::read(s).unwrap()[0], &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(Value::Pattern(p)) => panic!("Pattern!! {:?}", p),
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
