mod audio_device;
mod conflisp;
mod time;
mod units;

use std::sync::{Arc, Mutex, MutexGuard};

use audio_device::AudioDevice;
use time::Time;
use time::Clock;

use units::unit::Signal;
use units::unit::Unit;
use units::unit::Osc;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time::new(sample_rate, 120.0);

    let s = String::from("(wavetable (sine 0 1) (phase (saw 0 440)))");
    let unit_graph = units::ulisp::eval_one(&conflisp::read(s)[0]);

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.lock().unwrap().calc(&time).0 as f32;
            unit_graph.lock().unwrap().update(&time);
            time.inc();
        }
    });
}
