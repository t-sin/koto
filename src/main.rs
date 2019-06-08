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

    static mut table: [f64; 256] = [0.0f64; 256];
    let mut unit_graph;
    unsafe {
        for (i, e) in table.iter_mut().enumerate() {
            let th = (i as f64) / 256.0 * 3.141592 * 2.0;
            *e = th.sin();
        }
        unit_graph = Unit::Unit(Box::new(WaveTable {
            table: &table,
            len: 256,
            ph: unit_graph1,
        }));
    }

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.calc(&time).0 as f32;
            unit_graph.update(&time);
            time.update();
        }
    });
}
