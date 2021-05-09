use async_std::{
    channel::{self, Receiver, Sender},
    task,
};
use cpal::{
    self, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, OutputCallbackInfo, Sample, SampleFormat, Stream, StreamConfig,
};
use dashmap::DashSet;
use once_cell::sync::Lazy;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
};

static HOST: Lazy<Host> = Lazy::new(default_host);
static DEVICE: Lazy<Device> = Lazy::new(|| HOST.default_output_device().unwrap());
static CONFIG: Lazy<(StreamConfig, SampleFormat)> = Lazy::new(|| {
    let config = DEVICE.default_output_config().unwrap();
    let sample_format = config.sample_format();
    (
        DEVICE.default_output_config().unwrap().into(),
        sample_format,
    )
});
static CHANNELS: Lazy<usize> = Lazy::new(|| CONFIG.0.channels as usize);
static SAMPLE_RATE: Lazy<f32> = Lazy::new(|| CONFIG.0.sample_rate.0 as f32);
static CHANNEL: Lazy<(Sender<(u32, ElementState)>, Receiver<(u32, ElementState)>)> =
    Lazy::new(channel::unbounded::<(u32, ElementState)>);
const MAX_SCANCODE: usize = 64;

static KEY_FREQUENCIES: Lazy<[f32; MAX_SCANCODE]> = Lazy::new(|| {
    let mut frequencies = [0.; MAX_SCANCODE];
    frequencies
        .iter_mut()
        .enumerate()
        .for_each(|(scancode, freq)| {
            *freq = scancode_to_frequency(scancode as i32);
        });
    frequencies
});
static KEYS_PRESSED: Lazy<DashSet<usize>> = Lazy::new(DashSet::<usize>::new);

fn main() {
    let sample_format = CONFIG.1;
    let stream = match sample_format {
        SampleFormat::I16 => create_stream::<i16>(),
        SampleFormat::U16 => create_stream::<u16>(),
        SampleFormat::F32 => create_stream::<f32>(),
    };
    stream.play().unwrap();
    let event_loop = EventLoop::new();
    event_loop.run(move |event: Event<_>, _, cf: &mut ControlFlow| {
        if let Event::DeviceEvent {
            event:
                DeviceEvent::Key(KeyboardInput {
                    state, scancode, ..
                }),
            ..
        } = event
        {
            if (scancode as usize) < MAX_SCANCODE {
                task::spawn(CHANNEL.0.send((scancode, state)));
            }
            *cf = ControlFlow::Wait;
        }
    });
}

fn create_stream<T: Sample>() -> Stream {
    task::spawn(async {
        while let Ok((scancode, element_state)) = CHANNEL.1.recv().await {
            match element_state {
                ElementState::Pressed => {
                    KEYS_PRESSED.insert(scancode as usize);
                }
                ElementState::Released => {
                    KEYS_PRESSED.remove(&(scancode as usize));
                }
            }
        }
    });
    DEVICE
        .build_output_stream(
            &CONFIG.0,
            |data: &mut [T], _: &OutputCallbackInfo| {
                let mut sample_clock = 0f32;
                for frame in data.chunks_mut(*CHANNELS) {
                    sample_clock = (sample_clock + 1.0) % *SAMPLE_RATE;
                    let number_of_keys_pressed = KEYS_PRESSED.len();
                    let next_value = if number_of_keys_pressed > 0 {
                        KEYS_PRESSED
                            .iter()
                            .map(|scancode| {
                                (sample_clock * KEY_FREQUENCIES[*scancode] * std::f32::consts::TAU
                                    / *SAMPLE_RATE)
                                    .sin()
                            })
                            .sum::<f32>()
                            / number_of_keys_pressed as f32
                    } else {
                        0.
                    };
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

fn scancode_to_frequency(scancode: i32) -> f32 {
    let scancode = scancode as i32;
    let row = (53 - scancode) / 13;
    let column = (scancode - 5 + row) % 13 - row;
    100.0 * 1.5f32.powi(row) * (4.0f32 / 3.).powi(column)
}
