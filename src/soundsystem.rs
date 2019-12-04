use cpal::Device;
use cpal::EventLoop;
use cpal::SampleFormat;
use cpal::SampleRate;
use cpal::OutputBuffer;
use cpal::UnknownTypeOutputBuffer;

use super::mtime::{Clock, Time};
use super::units::unit::{Unit, AUnit};

pub struct AudioDevice {
    pub event_loop: EventLoop,
    pub device: Device,
}

impl AudioDevice {
    pub fn open(sample_rate: u32) -> AudioDevice {
        let device = cpal::default_output_device().unwrap();
        let format = cpal::Format {
            channels: 2,
            sample_rate: SampleRate(sample_rate as u32),
            data_type: SampleFormat::F32,
        };
        let event_loop = EventLoop::new();
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

        let audio_device = AudioDevice {
            event_loop: event_loop,
            device: device,
        };
        audio_device.event_loop.play_stream(stream_id);

        audio_device
    }

    pub fn run<F: FnMut(OutputBuffer<f32>) + Send>(&self, mut callback: F) {
        self.event_loop.run(move |_stream_id, stream_data| {
            match stream_data {
                cpal::StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(buffer) }
                => callback(buffer),
                _ => (),
            }
        });

    }
}

pub struct SoundSystem {
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
