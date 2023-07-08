// Overhaul of these structs is heavily ispired by the way Fyrox Engine uses tinyaudio crate
// https://github.com/FyroxEngine/Fyrox/blob/a468028c8e65e057608483710a0da4d7cbf31cfc/fyrox-sound/src/engine.rs#L26

use wasm_bindgen::prelude::*;
use tinyaudio;

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

mod audio_utils;
#[derive(Clone, Copy)]

#[derive(Debug)]
struct ShaperNode {
    wave_pos:f32,
    amplitude:f32
}

struct AudioState {
    audio_device:Option<Box<dyn tinyaudio::BaseAudioOutputDevice>>,
    wave:Vec<ShaperNode>,
}

impl AudioState{
    pub fn new() -> AudioState {
        AudioState{ 
            audio_device:None,
            wave: vec![
                // min and max x vals for a wave, both set to zero so that the wave loops cleanly
                ShaperNode{wave_pos:0f32, amplitude:0f32}, // min value for x in clip space
                ShaperNode{wave_pos:1f32, amplitude:0f32}, // max value for x in clip space
            ],
        }
    }

    pub fn render(&mut self, buf: &mut [(f32, f32)], params: tinyaudio::OutputDeviceParameters) {
        buf.fill((0.0, 0.0));

        let mut clock = 0f32;

        // used in the loop to determine the linear interpolation needed
        let next_inflection_sample = (self.wave[self.wave.len() - 1].wave_pos * buf.len() as f32) as usize;
        // TODO: Fill audio buffer based on nodes
        for curr_chan_tuple in 0..buf.len() {
            clock = (clock + 1.0) % params.sample_rate as f32;

            let mut value: f32 = 0.0;
            
            // keeping this here to show an example of how the sample clock can be used to generate a sine wave
            // WaveState::Sine => { value = (clock * 440.0 * 2.0 * std::f32::consts::PI / params.sample_rate as f32).sin(); },
            
            // setting the left and right channels
            buf[curr_chan_tuple].0 = value;
            buf[curr_chan_tuple].1 = value;
        }
    }

    fn add_node_to_wave(&mut self, wave_pos:f32, amplitude:f32) {
        // inserts the new node where it belongs in x-coord increasing order so that wave can be rendered
        for i in 0..self.wave.len() {
            if wave_pos < self.wave[i].wave_pos {
                self.wave.insert(i, ShaperNode { wave_pos, amplitude });
                log::warn!("new node added to wave at {:?} wave state is now {:?}", &self.wave[i].wave_pos, &self.wave);
                break;
            }
        }
        // tells the sound engine to change the function that produces samples of our drawn waveform
    }
}

/// Sound engine manages contexts, feeds output device with data. Sound engine instance can be cloned,
/// however this is always a "shallow" clone, because actual sound engine data is wrapped in Arc.
#[derive(Clone)]
pub struct SoundEngine(Arc<Mutex<AudioState>>);

impl SoundEngine {
    /// Creates new instance of the sound engine. It is possible to have multiple engines running at
    /// the same time, but you shouldn't do this because you can create multiple contexts which
    /// should cover 99% of use cases.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let engine = Self::without_device();
        engine.initialize_audio_output_device()?;
        Ok(engine)
    }

    /// Creates new instance of a sound engine without OS audio output device (so called headless mode).
    /// The user should periodically run [`State::render`] if they want to implement their own sample sending
    /// method to an output device (or a file, etc.).
    pub fn without_device() -> Self {
        Self(Arc::new(Mutex::new(AudioState::new())))
    }

    /// Tries to initialize default audio output device.
    pub fn initialize_audio_output_device(&self) -> Result<(), Box<dyn Error>> {
        let state = self.clone();

        let params: tinyaudio::OutputDeviceParameters = tinyaudio::OutputDeviceParameters {
            channels_count: 2,
            sample_rate: 44100,
            channel_sample_count: 4410,
        };

        // TODO: figure out rendering
        // It looks like there is a separate mix buffer that this renderer writes from
        // figure out how to write to that buffer I guess?
        let device = tinyaudio::run_output_device( params,
            {
                move |buf| {
                    // SAFETY: This is safe as long as channels count above is 2.
                    let data = unsafe {
                        std::slice::from_raw_parts_mut(
                            buf.as_mut_ptr() as *mut (f32, f32),
                            buf.len() / 2,
                        )
                    };

                    state.state().render(data, params);
                }
            },
        )?;

        self.state().audio_device = Some(device);

        Ok(())
    }

    /// Destroys current audio output device (if any).
    pub fn destroy_audio_output_device(&self) {
        self.state().audio_device = None;
    }

    /// Provides direct access to actual engine data.
    pub fn state(&self) -> MutexGuard<AudioState> {
        self.0.lock().unwrap()
    }

    pub fn add_node(&mut self, wave_pos:f32, amplitude:f32){
        self.state().add_node_to_wave(wave_pos, amplitude);
    }

    pub fn handle_audio_maintenance_events(&mut self, event: &Event<()>, control_flow: &mut ControlFlow){
        match event {
            Event::WindowEvent {event,..} => {
                match event {
                    WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                        if button == &winit::event::MouseButton::Left && state == &winit::event::ElementState::Pressed {
                            //TODO: this is sloppy, avoids recursive mutex unlock though
                            let mut already_init:bool = false;
                            match &self.state().audio_device {
                                None => { already_init = false },
                                Some(device) => { already_init = true }
                            }
                            if !already_init {
                                self.initialize_audio_output_device();
                            } else {
                                log::warn!("Hello from state changing");
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