mod rendering;
mod audio;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use web_sys;
use wasm_bindgen::prelude::*;
use tinyaudio::prelude::*;

pub struct ShaperState {
    render_state:rendering::State,
    audio_state:audio::AudioState
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {

    // setup rendering state and attach a window that can be rendered to
    let (mut event_loop, mut render_state) = rendering::State::init_rendering().await;
    let mut audio_state = audio::AudioState::new();

    // setup the audio waveshaper state and store it in this struct
    let mut program_state = ShaperState{
        render_state,
        audio_state
    };
    
    event_loop.run( move |event, _, control_flow| {
        program_state.render_state.handle_rendering_events(&event, control_flow);
        program_state.audio_state.handle_audio_events(&event, control_flow);
    });
    
}