mod audio_device;
mod time;
mod event;
mod units;
mod tapirlisp;

use audio_device::AudioDevice;
use time::{Time, Clock};

use units::unit::Unit;
use units::sequencer::{AdsrEg, Seq};

use tapirlisp::types::Value;
use tapirlisp as tlisp;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time::new(sample_rate, 120.0);

    let s = String::from("(wavetable (saw 0 1) (phase (saw 0 440)))");
    let osc = match tlisp::eval(&tlisp::read(s).unwrap()[0]) {
        Ok(Value::Unit(osc)) => osc,
        _err => return,
    };

    let eg = AdsrEg::new(1000, 10000, 0.5, 10000);
    let s2 = String::from(r"(pat ((c 2) (r 2)   (d 2) (r 2)
                                  (e 2) (r 2)   (f 2) (r 2)
                                  (g 2) (r 2)   (a 2) (r 2)
                                  (b 2) (r 2)   (c5 2) (r 2)))");
    let pat = match tlisp::eval(&tlisp::read(s2).unwrap()[0]) {
        Ok(Value::Pattern(pat)) => pat,
        _err => return,
    };
    println!("{:?}", pat);
    let unit_graph = Seq::new(pat, osc, eg);

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.lock().unwrap().calc(&time).0 as f32;
            unit_graph.lock().unwrap().update(&time);
            time.inc();
        }
    });
}
