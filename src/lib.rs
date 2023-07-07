mod rendering;
mod audio;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use web_sys;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
struct ShaperNode {
    clip_pos:[f32;4],
}

pub struct ShaperState {
    render_state:rendering::State,
    sound_engine:audio::SoundEngine,
    wave:Vec<ShaperNode>,
}
impl ShaperState {
    fn new(render_state:rendering::State, sound_engine:audio::SoundEngine) -> ShaperState {
        let default_wave = vec![
            ShaperNode{clip_pos:[-1f32, 0f32, 0f32, 1f32]}, // min value for x in clip space
            ShaperNode{clip_pos:[ 1f32, 0f32, 0f32, 1f32]}, // max value for x in clip space
        ];
        ShaperState {
            render_state,
            sound_engine,
            wave:default_wave
        }
    }

    // TODO: add node to the waveshaper
    fn add_node_to_wave(&mut self, new_node:ShaperNode){
        match self.wave.len() {
            0 => {log::warn!("Something is wrong, the wave should not be empty")},
            _ => {
                // inserts the new node where it belongs in x-coord increasing order so that wave can be rendered
                for i in 0..self.wave.len() {
                    if &new_node.clip_pos[0] < &self.wave[i].clip_pos[0] {
                        self.wave.insert(i, new_node);
                        log::warn!("new node added to wave at {:?} wave state is now {:?}", &self.wave[i].clip_pos, &self.wave);
                        break;
                    }
                }
                // tells the sound engine to change the function that produces samples of our drawn waveform
            }
        }
    }

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
                                self.add_node_to_wave(ShaperNode { clip_pos:new_node_loc });
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