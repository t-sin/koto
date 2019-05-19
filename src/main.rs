
use cpal::EventLoop;
use cpal::SampleFormat;
use cpal::SampleRate;

fn main() {
    let sample_rate = 44100;

    let device = cpal::default_output_device().unwrap();
    let format = cpal::Format {
        channels: 1,
        sample_rate: SampleRate(sample_rate),
        data_type: SampleFormat::F32
    };
    let event_loop = EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

    println!("formats: {:?}", format);
    event_loop.play_stream(stream_id);

    let mut ph = 0.0f64;

    event_loop.run(move |_stream_id, stream_data| {
        match stream_data {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for elem in buffer.iter_mut() {
                    *elem = ph.sin() as f32;
                    ph += (440.0 / sample_rate as f64) * std::f64::consts::PI;
                }
            }
            _ => ()
        }
    });
}
