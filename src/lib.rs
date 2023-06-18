mod rendering;
mod audio;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use web_sys;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {

    // setup rendering state and attach a window that can be rendered to
    let (mut event_loop, mut render_state) = rendering::State::init_rendering().await;
    
    event_loop.run( move |event, _, control_flow| {
        // TODO: write stuff for handling event for citation graph
        
        render_state.handle_rendering_events(event, control_flow);
    });
}

#[wasm_bindgen]
pub fn play_sine_wave() {
    log::warn!("Hello From Wasm Function");

    audio::init_audio();
}