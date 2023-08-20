// Overhaul of these structs is heavily ispired by the way Fyrox Engine uses tinyaudio crate
// https://github.com/FyroxEngine/Fyrox/blob/a468028c8e65e057608483710a0da4d7cbf31cfc/fyrox-sound/src/engine.rs#L26

use wasm_bindgen::prelude::*;
use tinyaudio;

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::usize;

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
        Wave { node_list:vec![init_node], curr_node_index:0, curr_node:init_node.clone(), interval_progress:0.0f32, freq_mult:2.0 }
    }

    pub fn set_freq(&mut self, new_freq:f32){
        self.freq_mult = new_freq.clamp(0.0, 100.0)
    }

    // Add a node to the wave
    fn insert_node(&mut self, new_node:WaveNode) { 
        // if empty list, populate the head, else search for place within list where this fits
        match self.node_list.is_empty() {
            true => self.node_list.push(new_node),
            false => {
                let res = self.node_list.binary_search_by(|probe| probe.wave_pos.total_cmp(&new_node.wave_pos));
                match res {
                    Ok(index) => {
                        // binary search was able to find an element at this exact position in the node list, don't add
                        log::warn!("Error: there is already a node at position: {} not adding node to list", new_node.wave_pos);
                    },
                    Err(index) => {
                        // binary search could not find a node at this wave position, tells us the index of where it 
                        // would be in the list if it existed, use that to insert the node and preserve sort by wave pos
                        self.node_list.insert(index, new_node);
                        log::warn!("node added at index: {}", index);
                    }
                }
            }
        }
    }

    fn peek_next_node(&self) -> &WaveNode{
        &self.node_list[(self.curr_node_index + 1) % self.node_list.len()]
    }

    fn incr_curr_node(&mut self) {
        self.curr_node_index = (self.curr_node_index + 1) % self.node_list.len();
        self.curr_node = self.node_list[self.curr_node_index];
    }

    fn interval_len_in_samples(&self, start_node:&WaveNode, end_node:&WaveNode, bufsize:usize) -> f32{
        let sample_len_of_wave = (bufsize as f32 / self.freq_mult);

        // get length of interval relative to the entire wave ( will be a fraction )
        let interval_rel_len = match end_node.wave_pos <= start_node.wave_pos {
            true => (end_node.wave_pos + 1.0) - start_node.wave_pos,
            false => end_node.wave_pos - start_node.wave_pos
        };

        // apply the freq multiplier to determine the number of samples in this interval
        (interval_rel_len * sample_len_of_wave) // recip, because increasing the freq should shorten the wave
    }

    fn interval_samples_remaining(intvl_sample_len:f32, curr_progress:f32) -> usize{
        ((1.0f32 - curr_progress) * intvl_sample_len) as usize
    }

    fn piecewise_linear(&mut self, buf: &mut [(f32, f32)]) -> f32 {

        if self.node_list.len() < 2 {
            return 0.0
        }

        let mut curr_sample: usize = 0;

        while curr_sample < buf.len() {
            // calculate the end index of this interval based on the play head and progress
            let interval_len_samples = Self::interval_len_in_samples(&self, &self.curr_node, self.peek_next_node(), buf.len());
            let intvl_samples_remain = Self::interval_samples_remaining(interval_len_samples, self.interval_progress);
            let end_sample = curr_sample + intvl_samples_remain;
            let progress_incr = 1.0 / interval_len_samples;

            //log::warn!("interval len: {interval_len_samples}\nrem_sample: {end_sample}\ncurr_sample: {curr_sample} \nprogress:{:?}", self.interval_progress);
            // do a while loop between the start and end points of this interval
            while curr_sample < buf.len() && curr_sample < end_sample {
                // calculate value of this index into the buffer
                let value = self.curr_node.amplitude * (1.0 - self.interval_progress) + self.peek_next_node().amplitude * self.interval_progress;
                buf[curr_sample].0 = value;
                buf[curr_sample].1 = value;

                // if curr_sample == end_sample - 1 {
                //     log::warn!("value at end of interval is: {value}\n progress is {:?}", self.interval_progress);
                // }

                self.interval_progress += progress_incr;
                curr_sample += 1;
            }

            if curr_sample < buf.len() {
                self.incr_curr_node();
                self.interval_progress = 0.0;
            }
        }
        
        // return the progress point of the next sample that fall outside this frame
        // it will be used as the offset for generating the next frame
        self.interval_progress // TODO: remove, this is just to shut the linter up
    }
}

impl std::fmt::Display for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!(); // TODO: implement
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

    pub fn set_new_freq_from_delta(&mut self, delta:f32) {
        // TODO: learn how the "cents" pitch measurement system works, just increaing the multiplier linearly makes it so the pitch goes up
        // a lot more with each step than it does in the lower registers. I want a smooth pitch transition
        let curr_freq = self.wave.as_mut().unwrap().freq_mult;
        self.wave.as_mut().unwrap().set_freq(curr_freq + delta);
    }

    pub fn render(&mut self, buf: &mut [(f32, f32)], params: tinyaudio::OutputDeviceParameters) {
        buf.fill((0.0, 0.0));
        
        // TODO: what is the system by which the user can switch between rendering techniques?

        // Fill audio buffer based on nodes in the Shaper Nodes vector
        // functions in the AudioBufGen module also return the progess point of the sample in the buffer
        // generated immediately after this one, this can be used as the offset for the next buffer
        // Self::piecewise_linear(buf, &mut self.play_state.as_mut().unwrap(), &self.freq_mult);

        // TODO: I don't really like that the rendering methods are a part of the "wave" structure

        self.wave.as_mut().unwrap().piecewise_linear(buf);
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

    pub fn print_node_list(&self) {
        log::warn!("state of audio node list is now: {:?}", self.state().wave.as_ref().unwrap().node_list)
    }

    pub fn apply_delta_to_frequency(&self, delta:f32){
        self.state().set_new_freq_from_delta(delta);
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