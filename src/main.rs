mod audio_device;
mod time;
mod event;
mod units;
mod tapirlisp;

use audio_device::AudioDevice;
use time::{Time, Clock};

use units::unit::{Unit, AUnit};

use tapirlisp::types::{Value, Env};
use tapirlisp as tlisp;

struct SoundSystem {
    time: Time,
    root_ug: AUnit,
}

impl SoundSystem {
    pub fn new(time: Time, ug: AUnit) -> SoundSystem {
        SoundSystem {
            time: time,
            root_ug: ug,
        }
    }

    pub fn run(&mut self, ad: &AudioDevice) {
        ad.run(|mut buffer| {
            let mut iter = buffer.iter_mut();
            loop {
                let (l, r) = self.root_ug.0.lock().unwrap().proc(&self.time);
                match iter.next() {
                    Some(lref) => *lref = l as f32,
                    None => break,
                }
                match iter.next() {
                    Some(rref) => *rref = r as f32,
                    None => break,
                }
                self.time.inc();
            }
        });
    }
}

fn main() {
    let sample_rate = 44100u32;
    let time = Time::new(sample_rate);

    let mut env = Env::init(time);
    let s = r"
(def $pat1 (pat (c4 1) (d 1) (e 1) (f 1) (g 1) (a 1) (b 1) (c5 1) (r 4)
                 loop))

(def $osc1 (wavetable (pulse 0 1 0.5)(phase (saw 0 440))))
(def $eg1 (adsr 0 (gain 0.2 (offset 1 (saw 0 0.25))) 0.0 0.1))

(def $pat2 (pat (c 2) (r 2) (c 2) (r 2) (c 2) (r 2) (c 2) (r 2) loop))
(def $osc2 (rand 0))
(def $eg2 (adsr 0 0.1 0.005 0))

(def $pat3 (pat (c3 2) (c3 2) (r 1) (c3 2) (c3 2) (c3 2) (r 1) (a+5 1) (r 1) (f5 1) (r 1) loop))
(def $osc3 (saw 0 440))
(def $eg3 (adsr 0.02 0.15 0.6 0))
(def $feg3 (adsr 0 0.1 0.3 0))

(+ (pan 0.2 (gain 0.15 (delay 0.3 0.5 1 (seq $pat1 $osc1 $eg1))))
   (gain 0.25 (seq $pat2 $osc2 $eg2))
   (gain 0.2 (lpf (+ 500 (+ 200 (* 200 (tri 0 0.5))) (* 1200 $feg3)) 10 (seq $pat3 $osc3 (trig $eg3 $feg3)))))".to_string();
    let unit_graph = match tlisp::eval_all(tlisp::read(s).unwrap(), &mut env) {
        Ok(Value::Unit(ug)) => ug,
        Ok(_v) => panic!("Oh, unit graph is not a unit!!"),
        Err(err) => panic!("Error!!! {:?}", err),
    };
    println!("{}", tlisp::dump(unit_graph.clone(), &env));

    let mut lcd = SoundSystem::new(env.time, unit_graph);

    let audio_device = AudioDevice::open(lcd.time.sample_rate);
    lcd.run(&audio_device);
}
