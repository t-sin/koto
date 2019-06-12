mod audio_device;
mod time;
mod units;

use std::sync::{Arc, Mutex};

use audio_device::AudioDevice;
use time::Time;
use time::Clock;

use units::unit::Signal;
use units::unit::Osc;
use units::unit::Unit;

use units::oscillator::WaveTable;
// use units::oscillator::TablePhase;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time::new(sample_rate, 120.0);

    // let s = String::from("(gain 0.5 (offset 1 (saw 0.0 440)))");
    // let unit_graph1 = units::conflisp::eval_one(&units::conflisp::read(s)[0]);

    // let mut table = Vec::new();
    // let mut unit_graph;
    // for i in 0..256 {
    //     let th = (i as f64) / (256.0) * std::f64::consts::PI * 2.0;
    //     table.push(th.sin());
    // }
    // unit_graph = Arc::new(Mutex::new(
    //     Unit::Unit(Arc::new(Mutex::new(WaveTable {
    //         table: table,
    //         ph: unit_graph1
    // })))));
    let s = String::from("(sine 0 440)");
    let mut unit_graph = units::conflisp::eval_one(&units::conflisp::read(s)[0]);
    // I want to set a freq!!!!!!!!!!!!
    unit_graph.lock().unwrap().set_freq(Arc::new(Mutex::new(Unit::Value(880.0))));

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.lock().unwrap().calc(&time).0 as f32;
            unit_graph.lock().unwrap().update(&time);
            time.inc();
        }
    });
}
