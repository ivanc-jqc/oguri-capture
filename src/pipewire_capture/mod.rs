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
use std::error::Error;
use std::io::SeekFrom;
use std::os::fd::OwnedFd;
use std::sync::mpsc;
pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 1;
pub const DEFAULT_VOLUME: f64 = 1.0;
pub const PI_2: f64 = std::f64::consts::PI + std::f64::consts::PI;
pub const CHAN_SIZE: usize = std::mem::size_of::<i16>();

struct AudioData<'a> {
    format: AudioInfoRaw,
    write_buffer: &'a mut CircularBuffer<f32>,
}

pub struct PipewireManager {
    pub main_loop: MainLoopRc,
    pub context: ContextRc,
    registry: RegistryRc,
    registry_listener: registry::Listener,
    pub audio_core: CoreRc,
    pub audio_buffer: CircularBuffer<f32>,

}
impl PipewireManager {
    pub fn new() -> Self {
        let mainloop = MainLoopRc::new(None).unwrap();
        let context = ContextRc::new(&mainloop, None).unwrap();
        let audio_core = context.connect_rc(None).unwrap();
        let registry = audio_core.get_registry_rc().unwrap();
        let reg_listener = registry.add_listener_local().global(|global| {
           dbg!(global);
        }).register();
        Self {
            main_loop: mainloop,
            context: context,
            registry: registry,
            registry_listener: reg_listener,
            audio_core: audio_core,
            audio_buffer: CircularBuffer::new((DEFAULT_RATE * 30).try_into().unwrap()),
        }
    }
    pub fn create_audio_stream(&mut self) {
        let audio_stream = create_audio_stream(self.audio_core.clone()).unwrap();
        let listener = attach_audio_listener(audio_stream.clone(), &mut self.audio_buffer).unwrap();
        let audio_stream_params_data = create_stream_params_bytes();
        let mut audio_stream_params = [Pod::from_bytes(&audio_stream_params_data).unwrap()];
        audio_stream.connect(
            Direction::Input,
            None, //which node to listen to
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut audio_stream_params,
        ).unwrap();
    }
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
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

fn attach_audio_listener<'a>(
    stream: StreamRc,
    write_buffer: &'a mut CircularBuffer<f32>,
) -> Result<StreamListener<AudioData<'a>>, Box<dyn Error>> {
    let audio_data = AudioData {
        format: Default::default(),
        write_buffer: write_buffer,
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
                audio_data.write_buffer.push(sample);
                println!("{}", sample);
            }
        })
        .register()?;
    Ok(listener)
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
