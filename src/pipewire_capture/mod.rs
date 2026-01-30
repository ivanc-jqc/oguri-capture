use crate::main;

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

    pub audio_core: CoreRc,
    pub audio_streams: RefCell<Vec<StreamRc>>,
    pub audio_listeners: RefCell<Vec<StreamListener<AudioData>>>,
    pub audio_buffer: Rc<RefCell<CircularBuffer<f32>>>,

    pub video_core: CoreRc
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
            audio_core: audio_core,
            audio_streams: RefCell::new(Vec::new()),
            audio_listeners: RefCell::new(Vec::new()),
            audio_buffer: Rc::new(RefCell::new(CircularBuffer::new(
                (DEFAULT_RATE * 30).try_into().unwrap()),
            )),
            video_core: video_core
        }
    }
    pub fn connect_audio(&self) -> Result<(), Box<dyn Error>> {
        let audio_stream = create_audio_stream(self.audio_core.clone()).unwrap();
        let listener = self.attach_audio_listener(audio_stream.clone()).unwrap();
        let audio_stream_params_data = create_stream_params_bytes();
        let mut audio_stream_params = [Pod::from_bytes(&audio_stream_params_data).unwrap()];
        audio_stream
            .connect(
                Direction::Input,
                None, //which node to listen to
                StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
                &mut audio_stream_params,
            )
            .unwrap();
        self.audio_streams.borrow_mut().push(audio_stream);
        self.audio_listeners.borrow_mut().push(listener);
        Ok(())
    }
    fn attach_audio_listener(
        &self,
        stream: StreamRc,
    ) -> Result<StreamListener<AudioData>, Box<dyn Error>> {
        dbg!("hi");
        let audio_data = AudioData {
            format: Default::default(),
            write_buffer: self.audio_buffer.clone(),
        };
        let listener = stream
            .add_local_listener_with_user_data(audio_data)
            .param_changed(|stream, user_data, id, param| {
                let Some(param) = param else {
                    return;
                };
            })
            .process(|stream, audio_data| match stream.dequeue_buffer() {
                None => println!("No buffer received"),
                Some(mut buffer) => {
                    let datas = buffer.datas_mut();
                    let mut data = &mut datas[0];
                    let bytes = data.data().unwrap();
                    let slice = &bytes[0..4];
                    let sample = f32::from_le_bytes(slice.try_into().unwrap());
                    audio_data.write_buffer.borrow_mut().push(sample);
                    println!("{}", sample);
                }
            })
            .register()?;
        Ok(listener)
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.main_loop.run();
        Ok(())
    }
}

fn create_audio_stream(audio_core: CoreRc) -> Result<StreamRc, Box<dyn Error>> {
    let stream = StreamRc::new(
        audio_core,
        "oguri-capture",
        properties! {
            "PW_KEY_MEDIA_TYPE" => "Audio",
            "PW_KEY_MEDIA_CATEGORY" => "Capture",
            // "PW_KEY_MEDIA_ROLE" => "Playback"
        },
    )?;
    Ok(stream)
}

fn create_stream_params_bytes() -> Vec<u8> {
    let mut audio_info = AudioInfoRaw::new();
    audio_info.set_format(AudioFormat::F32LE);
    audio_info.set_rate(DEFAULT_RATE);
    audio_info.set_channels(DEFAULT_CHANNELS);
    // let mut position = [0; pipewire::spa::param::audio::MAX_CHANNELS];
    // position[0] = libspa_sys::SPA_AUDIO_CHANNEL_FL;
    // position[1] = libspa_sys::SPA_AUDIO_CHANNEL_FR;
    // audio_info.set_position(position);

    let pod_object = Object {
        type_: pipewire::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
        id: ParamType::EnumFormat.as_raw(),
        properties: audio_info.into(),
    };

    let values: Vec<u8> = pipewire::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pipewire::spa::pod::Value::Object(pod_object),
    )
    .ok()
    .unwrap()
    .0
    .into_inner();

    // let mut params = [Pod::from_bytes(&values).unwrap()];
    values
}
