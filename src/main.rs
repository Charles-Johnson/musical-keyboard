use std::{
    io::{stdin, stdout, Read, Write},
    thread::sleep,
    time::Duration,
};

use cpal::{
    self, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, OutputCallbackInfo, Sample, SampleFormat, Stream, StreamConfig,
};
use once_cell::sync::Lazy;
use termion::raw::IntoRawMode;

static HOST: Lazy<Host> = Lazy::new(default_host);
static DEVICE: Lazy<Device> = Lazy::new(|| HOST.default_output_device().unwrap());

fn main() {
    let config = DEVICE.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0 as f32;
    let sample_format = config.sample_format();
    let stream_config: StreamConfig = config.into();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = stdin();
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
    let mut buffer = [0];
    loop {
        if let Ok(1) = stdin.read(&mut buffer) {
            let local_stream_config = stream_config.clone();
            let frequency = buffer[0] as f32 * 10.0;
            let stream = match sample_format {
                SampleFormat::I16 => {
                    create_stream::<i16>(&local_stream_config, sample_rate, frequency)
                }
                SampleFormat::U16 => {
                    create_stream::<u16>(&local_stream_config, sample_rate, frequency)
                }
                SampleFormat::F32 => {
                    create_stream::<f32>(&local_stream_config, sample_rate, frequency)
                }
            };
            stream.play().unwrap();
            sleep(Duration::from_millis(50));
            stream.pause().unwrap();
        }
    }
}

fn create_stream<T: Sample>(
    stream_config: &StreamConfig,
    sample_rate: f32,
    frequency: f32,
) -> Stream {
    let channels = stream_config.channels as usize;
    DEVICE
        .build_output_stream(
            &stream_config,
            move |data: &mut [T], _: &OutputCallbackInfo| {
                let mut sample_clock = 0f32;
                for frame in data.chunks_mut(channels) {
                    sample_clock = (sample_clock + 1.0) % sample_rate;
                    let next_value =
                        (sample_clock * frequency * std::f32::consts::TAU / sample_rate).sin();
                    let value = cpal::Sample::from::<f32>(&next_value);
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            |err| eprintln!("streaming error: {}", err),
        )
        .unwrap()
}
