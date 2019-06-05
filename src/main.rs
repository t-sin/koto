mod audio_device;
mod time;
mod units;

use std::io::{self, Read};

use audio_device::AudioDevice;
use time::Time;
use time::Clock;
use units::unit::Signal1;
use units::unit::Stateful;
use units::unit::Unit1;

use units::core::Offset;
use units::core::Gain;
use units::oscillator::Sine;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time { sample_rate: sample_rate, tick: 0 };

    let mut s = String::new();
    // I want to construct the unit graph like this:
    // let mut unit_graph = Unit1::Unit(Box::new(Sine {
    //     init_ph: Unit1::Value(0.0),
    //     ph: 0.0,
    //     freq: Unit1::Unit(Box::new(Offset {
    //         v: 880.0,
    //         src: Unit1::Unit(Box::new(Gain {
    //             v: 20.0,
    //             src: Unit1::Unit(Box::new(Sine {
    //                 init_ph: Unit1::Value(0.0),
    //                 ph: 0.0,
    //                 freq: Unit1::Value(20.0),
    //             })),
    //         })),
    //     })),
    // }));
    s.push_str("(sine 0.0 (offset 880.0 (gain 20.0 (sine 0.0 20.0))))");
    let sexp = units::conflisp::read(s);
    let mut unit_graph = units::conflisp::construct(sexp);

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.calc1(&time) as f32;
            unit_graph.update(&time);
            time.update();
        }
    });
}
