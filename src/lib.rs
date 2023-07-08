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
    sound_engine:audio::SoundEngine,
}
impl ShaperState {
    fn new(render_state:rendering::State, sound_engine:audio::SoundEngine) -> ShaperState {
        ShaperState {
            render_state,
            sound_engine,
        }
    }

    // TODO: holy shit this is so nested
    // handles interaction events that affect both the audio and visual states of the wave shaper
    fn handle_shaper_events(&mut self, event:&Event<()>, control_flow:&mut ControlFlow) {
        match event {
            Event::WindowEvent {event,..} => {
                match event {
                    WindowEvent::MouseInput { state, button,.. } => {
                        match button {
                            MouseButton::Left if state == &ElementState::Pressed => {
                                let new_node_loc = self.render_state.get_cursor_clip_location();
                                self.render_state.add_circle_at_clip_location(new_node_loc);
                                self.sound_engine.add_node( ((new_node_loc[0] + 1.0) / 2.0), new_node_loc[1]);
                            },
                            _ => {}
                        }
                        
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn handle_event(&mut self, event:&Event<()>, control_flow:&mut ControlFlow) {
        // If you ever need an input to be "consumed" at some point, this is the place to do it
        
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
    let mut program_state = ShaperState::new(
        render_state,
        sound_engine
    );
    
    event_loop.run( move |event, _, control_flow| {
        program_state.handle_event(&event, control_flow);
    });
    
}