
use cpal::EventLoop;
use cpal::SampleFormat;
use cpal::SampleRate;

mod clock;
mod unit;

use unit::Unit;

fn main() {
    let sample_rate = 44100.0;

    let device = cpal::default_output_device().unwrap();
    let format = cpal::Format {
        channels: 1,
        sample_rate: SampleRate(sample_rate as u32),
        data_type: SampleFormat::F32
    };
    let event_loop = EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

    event_loop.play_stream(stream_id);

    let clock = clock::Clock { tick: 0, sample_rate: sample_rate };
    let sine = unit::Osc { init_ph: 0.0, ph: 0.0, freq: 880.0 };

    event_loop.run(move |_stream_id, stream_data| {
        match stream_data {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for elem in buffer.iter_mut() {
                    *elem = sine.calc(&clock) as f32;
                }
            }
            _ => ()
        }
    });
}
