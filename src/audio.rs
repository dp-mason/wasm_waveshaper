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

#[derive(Clone, Copy, Debug)]
pub struct ShaperNode {
    wave_pos:f32,
    amplitude:f32
}

// module of functions that generate a wave within a buffer and return the offset of the next buffer
mod AudioBufGen {
    pub fn piecewise_linear(buf: &mut [(f32, f32)], wave:&Vec<super::ShaperNode>, wavelen:usize, frame_offset:f32) -> f32{
        let mut curr_sample = 0;
        let mut progress = 0.0;

        for node_index in 0..wave.len() - 1 {
            let mut interval_start = (wave[node_index    ].wave_pos * (buf.len() as f32)).floor() as usize;
            let mut interval_end   = (wave[node_index + 1].wave_pos * (buf.len() as f32)).floor() as usize;

            while curr_sample < interval_end {
                let mut value: f32 = 0.0;

                // Interpolates the amplitude of samples over a subsection of the wave marked by a start and end node
                // the frame offset and fract allow the wave to be generated over time independently of the buffer size
                progress = (((curr_sample - interval_start) as f32 / (interval_end - interval_start) as f32) + frame_offset).fract();
                value = wave[node_index].amplitude * (1.0f32 - progress) + wave[node_index + 1].amplitude * progress;
                
                // setting the left and right channels
                buf[curr_sample].0 = value;
                buf[curr_sample].1 = value;
                
                curr_sample = curr_sample + 1;
            }
        }
        // return the progress point of the next sample that fall outside this frame
        // it will be used as the offset for generating the next frame
        (progress + (1.0f32 / wavelen as f32)).fract()
    }
}



struct AudioState {
    audio_device:Option<Box<dyn tinyaudio::BaseAudioOutputDevice>>,
    wave:Vec<ShaperNode>,
    wavelen_samples:usize,
    frame_buf_offset:f32,
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
            wavelen_samples:1024,
            frame_buf_offset:0f32,
        }
    }

    pub fn render(&mut self, buf: &mut [(f32, f32)], params: tinyaudio::OutputDeviceParameters) {
        buf.fill((0.0, 0.0));
        
        // TODO: what is the system by which the user can switch between rendering techniques?

        // Fill audio buffer based on nodes in the Shaper Nodes vector
        // functions in the AudioBufGen module also return the progess point of the sample in the buffer
        // generated immediately after this one, this can be used as the offset for the next buffer
        self.frame_buf_offset = AudioBufGen::piecewise_linear(buf, &self.wave, self.wavelen_samples, self.frame_buf_offset);
    }

    fn add_node_to_wave(&mut self, wave_pos:f32, amplitude:f32) {
        // inserts the new node where it belongs in x-coord increasing order so that wave can be rendered
        for i in 0..self.wave.len() {
            if wave_pos < self.wave[i].wave_pos {
                self.wave.insert(i, ShaperNode { wave_pos, amplitude });
                log::warn!("new node added to wave at {:?}", &self.wave[i].wave_pos);
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
                                log::warn!("Siund engine initialized audio device");
                            } else {
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