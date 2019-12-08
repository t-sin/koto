use cpal::Device;
use cpal::EventLoop;
use cpal::SampleFormat;
use cpal::SampleRate;
use cpal::OutputBuffer;
use cpal::UnknownTypeOutputBuffer;

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
