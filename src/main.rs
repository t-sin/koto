mod audio_device;
mod tapirlisp;
mod time;
mod events;
mod units;

use audio_device::AudioDevice;
use time::{Time, Clock};

use tapirlisp as tlisp;

use events::elisp as elisp;

use units::unit::Unit;
use units::ulisp as ulisp;

use units::sequencer::{AdsrEg, Seq};

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time::new(sample_rate, 120.0);

    let s = String::from("(sine 0 440)");
    let osc = ulisp::eval_one(&tlisp::read(s)[0]);

    let eg = AdsrEg::new(1, 1000, 1.0, 1000);
    let s2 = String::from("((c4 2) (d4 0) (e4 0) (f4 0) (g4 0) (a4 0) (b4 0) (c5 0))");
    println!("{:?}", tlisp::print(&tlisp::read(s2.clone())[0]));
    let pat = elisp::eval_one(&tapirlisp::read(s2)[0]);
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
