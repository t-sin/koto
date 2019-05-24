mod audio_device;
mod time;
mod units;

use audio_device::AudioDevice;
use time::Time;
use time::Clock;
use units::unit::Unit;
use units::unit::UnitGraph;
use units::oscillator::Sine;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;

    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time { sample_rate: sample_rate, tick: 0 };
    let mut unit_graph = UnitGraph::Unit(Box::new(Sine { init_ph: 0.0, ph: 0.0, freq: 880.0 }));

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.calc(&time) as f32;
            unit_graph.update(&time);
            time.update();
        }
    });
}
