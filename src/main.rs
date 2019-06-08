mod audio_device;
mod time;
mod units;

use audio_device::AudioDevice;
use time::Time;
use time::Clock;
use units::unit::Signal;
use units::unit::Stateful;

use units::unit::Unit;
use units::oscillator::WaveTable;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time { sample_rate: sample_rate, tick: 0 };

    let mut s = String::from("(gain 0.5 (offset 1.0 (saw 0.0 440)))");
    let mut unit_graph1 = units::conflisp::eval_one(&units::conflisp::read(s)[0]);

    let mut table = Vec::new();
    let mut unit_graph;
    for i in 0..256 {
        let th = (i as f64) / (256.0) * std::f64::consts::PI * 2.0;
        table.push(th.sin());
    }
    unit_graph = Unit::Unit(Box::new(WaveTable {
        table: table,
        ph: unit_graph1,
    }));

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.calc(&time).0 as f32;
            unit_graph.update(&time);
            time.update();
        }
    });
}
