mod rendering;
mod audio;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use web_sys;
use wasm_bindgen::prelude::*;

pub struct ShaperState {
    render_state:rendering::State,
    sound_engine:audio::SoundEngine
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {

    // setup rendering state and attach a window that can be rendered to
    let (mut event_loop, mut render_state) = rendering::State::init_rendering().await;
    let mut sound_engine:audio::SoundEngine = audio::SoundEngine::without_device();

    // setup the audio waveshaper state and store it in this struct
    let mut program_state = ShaperState{
        render_state,
        sound_engine
    };
    
    event_loop.run( move |event, _, control_flow| {
        program_state.render_state.handle_rendering_events(&event, control_flow);
        program_state.sound_engine.handle_audio_events(&event, control_flow);
    });
    
}