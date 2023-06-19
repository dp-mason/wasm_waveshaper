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
    audio_device:Option<&'static mut dyn BaseAudioOutputDevice>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {

    // setup rendering state and attach a window that can be rendered to
    let (mut event_loop, mut render_state) = rendering::State::init_rendering().await;

    let mut program_state = ShaperState{
        render_state,
        audio_device:None
    };
    
    event_loop.run( move |event, _, control_flow| {
        program_state.render_state.handle_rendering_events(&event, control_flow);
        
        
        
        //TODO: move event handling logic somwhere else
        match event {
            Event::WindowEvent {event,..} => {
                match event {
                    WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                        if button == winit::event::MouseButton::Left {
                            match &program_state.audio_device {
                                None => program_state.audio_device = Some(audio::init_audio()),
                                Some(device) => {}
                            }
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        } 
        
    });
}

#[wasm_bindgen]
pub fn play_sine_wave() {
    //log::warn!("Hello From Wasm Function");
    audio::play_sine();
}