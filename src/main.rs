mod audio_device;
mod tapirlisp;
mod time;
mod units;

use audio_device::AudioDevice;
use time::{Time, Pos, Clock};

use units::unit::Unit;
use units::unit::{UnitGraph, UType, ADSR, Eg};
use units::sequencer::{AdsrEg, Event, Seq};

fn main() {
    let channels = 1;
    let sample_rate = 44100u32;
    let audio_device = AudioDevice::open(channels, sample_rate);

    let mut time = Time::new(sample_rate, 120.0);

    let s = String::from("(wavetable (saw 0 1) (phase (saw 0 440)))");
    let osc = units::ulisp::eval_one(&tapirlisp::read(s)[0]);

    let eg = AdsrEg::new(1, 1000, 1.0, 1000);
    let mut pat = Vec::new();
    pat.push(Box::new(Event::On(Pos {bar: 0, beat: 0, pos: 0.0}, 440.0)));
    pat.push(Box::new(Event::Off(Pos {bar: 0, beat:0 , pos: 0.25})));
    pat.push(Box::new(Event::On(Pos {bar: 0, beat: 1, pos: 0.0}, 440.0)));
    pat.push(Box::new(Event::Off(Pos {bar: 0, beat: 1 , pos: 0.25})));
    pat.push(Box::new(Event::Loop(Pos {bar: 1, beat :0 , pos: 0.0})));
    let unit_graph = Seq::new(pat, osc, eg);

    audio_device.run(|mut buffer| {
        for elem in buffer.iter_mut() {
            *elem = unit_graph.lock().unwrap().calc(&time).0 as f32;
            unit_graph.lock().unwrap().update(&time);
            time.inc();
        }
    });
}
