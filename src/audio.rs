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

#[derive(Clone, Debug, PartialEq)]
pub struct WaveNode {
    wave_pos:f32,
    amplitude:f32,
    next:Option<Box<WaveNode>>,
}

// cyclical linked list used to generate waves in audio rendering functions
struct Wave {
    head: Option<Box<WaveNode>>,
    len: usize,
}

impl Wave {
    // Create an empty linked list
    fn new() -> Self {
        Wave { head: None, len:0 }
    }

    // Check if the linked list is empty
    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    // Add a node to the wave
    fn add(&mut self, new_node:WaveNode) {
        
        // if empty list, populate the head, else search for place within list where this fits
        match &self.head {
            None => {
                // set the head to the new node
                self.head = Some(Box::new(new_node));
            },
            _ => {
                let mut curr_wavenode = self.head.as_mut().unwrap();
                
                while curr_wavenode.next.is_some() && new_node.wave_pos < curr_wavenode.wave_pos  {
                    curr_wavenode = curr_wavenode.next.as_mut().unwrap(); // this is safe bc the list is cyclic
                }
                
                let tmp_next_node = curr_wavenode.next.clone();

                // insert the new node after the current wavenode and before the next, will still work for appending to end
                curr_wavenode.next = Some(Box::new(WaveNode { 
                    next: tmp_next_node,
                    ..new_node
                }));
            }
        }

        self.len = self.len + 1;
    
    }
}

impl std::fmt::Display for Wave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref head) = self.head {
            let mut current = &**head;
            loop {
                write!(f, "NODE: wave pos:{} amplitude:{}\n", current.wave_pos, current.amplitude)?;

                if let Some(ref next) = current.next {
                    current = &**next;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}

struct WavePlayState {
    curr_node:Box<WaveNode>,
    interval_progress:f32,
}

// module of functions that generate a wave within a buffer and return the offset of the next buffer
mod AudioBufGen {
    use super::WavePlayState;

    
}



struct AudioState {
    audio_device:Option<Box<dyn tinyaudio::BaseAudioOutputDevice>>,
    wave:Wave,
    play_state:Option<WavePlayState>,
    freq_mult:f32,
}

impl AudioState{
    pub fn new() -> AudioState {
        AudioState{ 
            audio_device:None,
            wave: Wave::new(),
            play_state:None,
            freq_mult:1.5,
        }
    }

    // TODO: move these buffer populating funcs somewhere else for organization eventually
    //  the problem before was that rust will not allow you to leak private types and I dont
    //  wanna make fields publicly exposed to change yet. different impl block??
    fn piecewise_linear(buf: &mut [(f32, f32)], play_state:&mut WavePlayState, freq_mult:&f32) -> f32 {    
        let mut curr_sample = 0;

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
        
        // TODO: I would really like a system where I dont have to do a lookup within the wave node buffer for every sample
        // need time to consider how to design this

        while curr_sample < buf.len() {
            // the wave is generated independently of the size of the buffer
            // for this reason we need to divide the buffer into "chunks" that may contain the enitery of the wave
            //  only the first part of the wave, only the last part of the wave, or only the middle, depending on how long
            //  the wavelength is compared to the buffer length (determined by freq multiplier)

            // while curr_sample < interval_end {

            //     let mut value: f32 = 0.0;

            //     // Interpolates the amplitude of samples over a subsection of the wave marked by a start and end node
            //     // the frame offset and fract allow the wave to be generated over time independently of the buffer size

            //     // TODO: in this line I am mixing up the ideas of progess through the interval and progress through the wave
            //     // TODO: this is also fucked because the curr_sample trick worked when there was 1 wave per buffer
            //     let intrvl_progress = ( (curr_sample - interval_start) as f32 / (interval_end - interval_start) as f32 ).fract();
            //     value = wave[node_index].amplitude * (1.0f32 - intrvl_progress) + wave[node_index + 1].amplitude * intrvl_progress;
                
            //     // setting the left and right channels
            //     buf[curr_sample].0 = value;
            //     buf[curr_sample].1 = value;
                
            //     curr_sample = curr_sample + 1;
            // }
        }

        // return the progress point of the next sample that fall outside this frame
        // it will be used as the offset for generating the next frame
        6.9f32 // TODO: remove, this is just to shut the linter up
    }

    pub fn render(&mut self, buf: &mut [(f32, f32)], params: tinyaudio::OutputDeviceParameters) {
        buf.fill((0.0, 0.0));
        
        // TODO: what is the system by which the user can switch between rendering techniques?

        // Fill audio buffer based on nodes in the Shaper Nodes vector
        // functions in the AudioBufGen module also return the progess point of the sample in the buffer
        // generated immediately after this one, this can be used as the offset for the next buffer
        Self::piecewise_linear(buf, &mut self.play_state.as_mut().unwrap(), &self.freq_mult);
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
        self.state().wave.add(WaveNode { wave_pos, amplitude, next:None });
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