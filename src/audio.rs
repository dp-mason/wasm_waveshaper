use wasm_bindgen::prelude::*;
use tinyaudio::prelude::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

mod audio_utils;

// TODO: figure out if there is a way to move the waveshaping function outside of the init function and store it separately in the AudioState
pub struct AudioState {
    audio_device:Option<&'static mut dyn BaseAudioOutputDevice>,
}

impl AudioState{
    pub fn new() -> AudioState {
        AudioState{ audio_device:None }
    }

    // TODO: add new wave shape functions

    // TODO: modify this function so that it changes the waveshaping function of the audio state rather than creating a new device
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

    fn init_audio() -> &'static mut dyn BaseAudioOutputDevice {
        log::warn!("Hello from init_audio");

        audio_utils::set_panic_hook();

        let params = OutputDeviceParameters {
            channels_count: 2,
            sample_rate: 44100,
            channel_sample_count: 4410,
        };

        // TODO: silence is the default upon init

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

    pub fn handle_audio_events(&mut self, event: &Event<()>, control_flow: &mut ControlFlow){
        //TODO: move event handling logic somwhere else
        match event {
            Event::WindowEvent {event,..} => {
                match event {
                    WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                        if button == &winit::event::MouseButton::Left {
                            match &self.audio_device {
                                None => self.audio_device = Some(Self::init_audio()),
                                Some(device) => {
                                    // TODO: cycle though different wave shapes each click
                                }
                            }
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}