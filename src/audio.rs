use wasm_bindgen::prelude::*;
use tinyaudio::prelude::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

mod audio_utils;

pub fn play_sine() {
    log::warn!("Hello from play_sine");

    audio_utils::set_panic_hook();

    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };

    let device: Box<dyn BaseAudioOutputDevice> = run_output_device(params, {
        let mut clock = 0f32;
        move |data| {
            for samples in data.chunks_mut(params.channels_count) {
                clock = (clock + 1.0) % params.sample_rate as f32;
                let value =
                    (clock * 440.0 * 2.0 * std::f32::consts::PI / params.sample_rate as f32).sin();
                for sample in samples {
                    *sample = value;
                }
            }
        }
    })
    .unwrap();

    Box::leak(device);
}

pub fn init_audio() -> &'static mut dyn BaseAudioOutputDevice {
    log::warn!("Hello from init_audio");

    audio_utils::set_panic_hook();

    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };

    let device: Box<dyn BaseAudioOutputDevice> = run_output_device(params, {
        let mut clock = 0f32;
        move |data| {
            for samples in data.chunks_mut(params.channels_count) {
                clock = (clock + 1.0) % params.sample_rate as f32;
                let value =
                    (clock * 440.0 * 2.0 * std::f32::consts::PI / params.sample_rate as f32).sin();
                for sample in samples {
                    *sample = value;
                }
            }
        }
    })
    .unwrap();

    return Box::leak(device);
}

pub fn handle_audio_events(event: Event<()>, control_flow: &mut ControlFlow){

}