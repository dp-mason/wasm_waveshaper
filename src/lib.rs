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
impl ShaperState {
    // TODO: add node to the waveshaper
    fn add_node_to_wave(){}

    // handles events that affect both the audio and visual states of the wave shaper
    fn handle_shaper_events(&mut self, event:&Event<()>, control_flow:&mut ControlFlow) {
        match event {
            Event::WindowEvent {event,..} => {
                match event {
                    WindowEvent::MouseInput { state, button,.. } => {
                        self.render_state.add_circle_at_cursor_location(state, button);
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn handle_event(&mut self, event:&Event<()>, control_flow:&mut ControlFlow) {
        // kinda sexy, handles window resizes, UI stuff that doesn't affect the sound of the wave shaper,
        self.render_state.handle_window_maintenance_events(event, control_flow);
        // not sexy, just ensuring audio state is constructed and maintained
        self.sound_engine.handle_audio_maintenance_events(event, control_flow);
        // sexy, handles events that change both the visual and audible state of the shaper
        self.handle_shaper_events(event, control_flow);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {

    // setup rendering state and attach a window that can be rendered to
    let (event_loop, render_state) = rendering::State::init_rendering().await;
    let sound_engine:audio::SoundEngine = audio::SoundEngine::without_device();

    // setup the audio waveshaper state and store it in this struct
    let mut program_state = ShaperState{
        render_state,
        sound_engine
    };
    
    event_loop.run( move |event, _, control_flow| {
        program_state.handle_event(&event, control_flow);
    });
    
}