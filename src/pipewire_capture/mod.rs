use super::circular_buffer::CircularBuffer;
use pipewire::context::{self, ContextRc};
use pipewire::core::{CoreBox, CoreRc, Listener};
use pipewire::properties::properties;
use pipewire::registry::{self, RegistryRc};
use pipewire::spa::{
    param::{
        ParamType,
        audio::{AudioFormat, AudioInfoRaw},
        video::{VideoFormat, VideoInfoRaw},
    },
    pod::{Object, Pod},
    utils::Direction,
};
use pipewire::stream::{StreamListener, StreamRc};
use pipewire::{
    context::ContextBox,
    main_loop::MainLoopRc,
    stream::{StreamBox, StreamFlags},
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::io::SeekFrom;
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::sync::mpsc;
use audio::Audio;
use video::Video;
pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 1;
pub const DEFAULT_VOLUME: f64 = 1.0;
pub const PI_2: f64 = std::f64::consts::PI + std::f64::consts::PI;
pub const CHAN_SIZE: usize = std::mem::size_of::<i16>();

pub struct AudioData {
    format: AudioInfoRaw,
    write_buffer: Rc<RefCell<CircularBuffer<f32>>>,
}

pub struct PipewireManager {
    pub main_loop: MainLoopRc,
    pub context: ContextRc,
    registry: RegistryRc,
    registry_listener: registry::Listener,

    pub audio: Audio,
    pub video: Video,
}

impl PipewireManager {
    pub fn new(fd: OwnedFd) -> Self {
        let mainloop = MainLoopRc::new(None).unwrap();
        let context = ContextRc::new(&mainloop, None).unwrap();
        let audio_core = context.connect_rc(None).unwrap();
        let video_core = context.connect_fd_rc(fd, None).unwrap();
        let registry = audio_core.get_registry_rc().unwrap();
        let reg_listener = registry
            .add_listener_local()
            .global(|global| {
                // dbg!(global);
            })
            .register();
        Self {
            main_loop: mainloop,
            context: context,
            registry: registry,
            registry_listener: reg_listener,
            audio: Audio::new(audio_core),
            video: Video::new(video_core),
        }
    }
    
    

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.main_loop.run();
        Ok(())
    }
}



mod audio;
mod video;