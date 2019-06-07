mod audio_device;
mod time;
mod units;

use audio_device::AudioDevice;
use time::Time;
use time::Clock;
use units::unit::Signal;
use units::unit::Stateful;

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time { sample_rate: sample_rate, tick: 0 };

    let mut s = String::from("(sine 0.0 (offset 880.0 (gain 20.0 (sine 0.0 20.0))))");
    let sexp = units::conflisp::read(s);
    println!("sexp: {:?}", units::conflisp::print(&sexp[0]));
    let mut unit_graph = units::conflisp::eval_one(&sexp[0]);

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.calc(&time).0 as f32;
            unit_graph.update(&time);
            time.update();
        }
    });
}
