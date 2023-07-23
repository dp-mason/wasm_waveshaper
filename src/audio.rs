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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WaveNode {
    wave_pos:f32,
    amplitude:f32,
}

struct Wave {
    node_list:Vec<WaveNode>,
    curr_node_index:usize,
    curr_node:WaveNode,
    interval_progress:f32,
    freq_mult:f32,
}

impl Wave {
    // Create an empty linked list
    fn new(init_node:WaveNode) -> Self {
        Wave { node_list:vec![init_node], curr_node_index:0, curr_node:init_node.clone(), interval_progress:0.0f32, freq_mult:1.0 }
    }

    // Add a node to the wave
    fn insert_node(&mut self, new_node:WaveNode) {
        
        // if empty list, populate the head, else search for place within list where this fits
        match self.node_list.is_empty() {
            true => self.node_list.push(new_node),
            false => {
                let mut index:usize = 0;
                while index < self.node_list.len() && new_node.wave_pos < self.node_list[index].wave_pos {
                    index += 1
                }
                self.node_list.insert(index, new_node);
                // todo: binary search so inseartion is a lil faster self.node_list.binary_search(
            }
        }

    }

    fn peek_next_node(&self) -> &WaveNode{
        &self.node_list[self.curr_node_index + 1 % self.node_list.len()]
    }

    fn incr_curr_node(&mut self) {
        self.curr_node_index += 1 % self.node_list.len();
        self.curr_node = self.node_list[self.curr_node_index];
    }

    fn interval_len_in_samples(&self, start_node:&WaveNode, end_node:&WaveNode, bufsize:usize) -> usize{
        // get length of interval relative to the entire wave ( will be a fraction )
        let interval_rel_len = match end_node.wave_pos < start_node.wave_pos {
            true => self.peek_next_node().wave_pos + 1.0 - start_node.wave_pos,
            false => self.peek_next_node().wave_pos - start_node.wave_pos
        };

        // apply the freq multiplier to determine the number of samples in this interval
        ((interval_rel_len * self.freq_mult.recip()) * bufsize as f32) as usize // recip, because increasing the freq should shorten the wave
    }

    fn piecewise_linear(&mut self, buf: &mut [(f32, f32)]) -> f32 {    
        
        // TODO: since the wave does not necessarily span the whole buffer anymore, this loop needs refactoring
        
        // for node_index in 0..wave.len() - 1 {
            //     let mut interval_start = (wave[node_index    ].wave_pos * (buf.len() as f32)).floor() as usize;
            //     let mut interval_end   = (wave[node_index + 1].wave_pos * (buf.len() as f32)).floor() as usize;
            
            //     while curr_sample < interval_end {
                //         let mut value: f32 = 0.0;
                
                //         // Interpolates the amplitude of samples over a subsection of the wave marked by a start and end node
                //         // the frame offset and fract allow the wave to be generated over time independently of the buffer size
                //         progress = (((curr_sample - interval_start) as f32 / (interval_end - interval_start) as f32) * freq_mult + frame_offset).fract();
        //         value = wave[node_index].amplitude * (1.0f32 - progress) + wave[node_index + 1].amplitude * progress;
                
        //         // setting the left and right channels
        //         buf[curr_sample].0 = value;
        //         buf[curr_sample].1 = value;
        
        //         curr_sample = curr_sample + 1;
        //     }
        // }
        
        let mut curr_sample: usize = 0;

        while curr_sample < buf.len() {
            // calculate the end index of this interval based on the play head and progress
            let interval_len_samples = Self::interval_len_in_samples(&self, &self.curr_node, self.peek_next_node(), buf.len());
            let progress_incr = 1.0 / interval_len_samples as f32;
            // do a while loop between the start and end points of this interval
            while curr_sample < buf.len() && self.interval_progress <= 1.0 {
                // calculate value of this index into the buffer
                let value = self.curr_node.amplitude * (1.0 - self.interval_progress) + self.peek_next_node().amplitude * self.interval_progress;
                buf[curr_sample].0 = value;
                buf[curr_sample].1 = value;

                self.interval_progress += progress_incr;
                curr_sample += 1;
            }

            if self.interval_progress >= 1.0 {
                self.incr_curr_node()
            }
        }
        
        // return the progress point of the next sample that fall outside this frame
        // it will be used as the offset for generating the next frame
        self.interval_progress // TODO: remove, this is just to shut the linter up
    }
}

impl std::fmt::Display for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // if let Some(ref head) = self.head {
        //     write!(f, "Printing wave with nodes:")?;
        //     let mut current = &**head;
        //     loop {
        //         write!(f, "     NODE: wave pos:{} amplitude:{}\n", current.wave_pos, current.amplitude)?;

        //         if let Some(ref next) = current.next {
        //             current = &**next;
        //         } else {
        //             break;
        //         }
        //     }
        // }
        // TODO: implement
        Ok(())
    }
}



struct AudioState {
    audio_device:Option<Box<dyn tinyaudio::BaseAudioOutputDevice>>,
    wave:Option<Wave>,
}

impl AudioState{
    pub fn new() -> AudioState {
        AudioState{ 
            audio_device: None,
            wave: None,
        }
    }

    

    pub fn render(&mut self, buf: &mut [(f32, f32)], params: tinyaudio::OutputDeviceParameters) {
        buf.fill((0.0, 0.0));
        
        // TODO: what is the system by which the user can switch between rendering techniques?

        // Fill audio buffer based on nodes in the Shaper Nodes vector
        // functions in the AudioBufGen module also return the progess point of the sample in the buffer
        // generated immediately after this one, this can be used as the offset for the next buffer
        // Self::piecewise_linear(buf, &mut self.play_state.as_mut().unwrap(), &self.freq_mult);

        if self.wave.is_some() {
            self.wave.as_mut().unwrap().piecewise_linear(buf);
        }
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
        let mut wave_initialized = match self.state().wave {
            None => false,
            Some(_) => true
        };
        match wave_initialized {
            true => {
                self.state().wave.as_mut().unwrap().insert_node(WaveNode { wave_pos, amplitude });
            },
            false => {
                self.state().wave = Some(Wave::new(WaveNode { wave_pos, amplitude }));
            }
        }
        
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
                                log::warn!("Sound engine initialized audio device");
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