mod rendering;
mod audio;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder}, dpi::Position,
};

use web_sys;
use wasm_bindgen::prelude::*;

//converts an entire exported visual state to a format that the audio renderer can recognize as a wave shape
// based on the anchors in the visual state and whether the anchors are in scope
fn wave_shape_from_visual_state(visual_state:rendering::VisualState) -> Vec<audio::WaveNode> {
    // TODO: use filter pattern here in the spirit of functional programming
    return vec![]
}

//TODO: shaper state is basically the UI layer. Why does it "own" instances of AudioState and RenderState?

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
                        match (button, state) {
                            (MouseButton::Left, ElementState::Pressed) => {
                                let new_node_loc = self.render_state.get_cursor_clip_location();
                                self.render_state.add_circle_at_clip_location(new_node_loc);
                                self.sound_engine.add_node( ((new_node_loc[0] + 1.0) / 2.0), new_node_loc[1]);
                                self.sound_engine.print_node_list();
                            },
                            _ => {}
                        }
                    },
                    WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => { 
                        self.sound_engine.apply_delta_to_frequency(match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                y * 0.001 //TODO: remove magic number
                            },
                            MouseScrollDelta::PixelDelta(pos) => {
                                pos.y as f32 * 0.001 //TODO: remove magic number
                            },
                        });
                    },
                    WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                        // KEYBOARD INPUT SECTION
                        match (input.virtual_keycode, input.state) {
                            (Some(VirtualKeyCode::S), ElementState::Pressed) => {
                                self.render_state.update_world_scale(self.render_state.get_world_scale() * 0.3);
                            },
                            (Some(VirtualKeyCode::R), ElementState::Pressed) => {
                                // export the current visual state, load the audio state with a new wave based on the exported state
                                
                            }
                            _ => {},
                        }
                    }
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