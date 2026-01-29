use super::circular_buffer::CircularBuffer;
use pipewire::core::CoreBox;
use pipewire::properties::properties;
use pipewire::spa::{
    param::{
        ParamType,
        audio::{AudioFormat, AudioInfoRaw},
        video::{VideoFormat, VideoInfoRaw},
    },
    pod::{Object, Pod},
    utils::Direction,
};
use pipewire::{
    context::ContextBox,
    main_loop::MainLoopBox,
    stream::{StreamBox, StreamFlags},
};
use std::error::Error;
use std::io::SeekFrom;
use std::os::fd::OwnedFd;
pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 1;
pub const DEFAULT_VOLUME: f64 = 1.0;
pub const PI_2: f64 = std::f64::consts::PI + std::f64::consts::PI;
pub const CHAN_SIZE: usize = std::mem::size_of::<i16>();

struct AudioData {
    format: AudioInfoRaw,
    write_buffer: CircularBuffer<f32>,
}

struct pw_handler<'a> {
    pub main_loop: MainLoopBox,
    context: Option<ContextBox<'a>>,
    core: Option<CoreBox<'a>>,
    file_descriptor: Option<OwnedFd>,
}

impl<'a> pw_handler<'a> {
    pub fn new(fd: OwnedFd) -> Result<Self, Box<dyn Error>> {
        let mainloop = MainLoopBox::new(None)?;

        Ok(Self {
            main_loop: mainloop,
            context: None,
            core: None,
            file_descriptor: Some(fd),
        })
    }
    pub fn init(&'a mut self) -> Result<(), Box<dyn Error>> {
        let context: ContextBox<'a>  = ContextBox::new(self.main_loop.loop_(), None)?;
        self.context = Some(context);
        let core: CoreBox<'a>  = self.context.as_ref().unwrap().connect(None)?;
        self.core = Some(core);
        Ok(())
    }
}

pub fn pw_init(fd: OwnedFd) -> Result<(), Box<dyn Error>> {
    let mainloop = MainLoopBox::new(None)?;
    let context = ContextBox::new(&mainloop.loop_(), None)?;
    // let video_core = context.connect_fd(fd, None)?;
    let video_core = context.connect(None)?;

    let audio_data = AudioData {
        format: Default::default(),
        write_buffer: CircularBuffer::new((DEFAULT_RATE * 30).try_into().unwrap()),
    };

    let stream = StreamBox::new(
        &video_core,
        "oguri-capture",
        properties! {
            "PW_KEY_MEDIA_TYPE" => "Audio",
            "PW_KEY_MEDIA_CATEGORY" => "Capture",
            // "PW_KEY_MEDIA_ROLE" => "Playback"
        },
    )?;
    let _video_listener = stream
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

    let mut params = [Pod::from_bytes(&values).unwrap()];

    stream.connect(
        Direction::Input,
        None,
        StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
        &mut params,
    )?;
    mainloop.run();
    Ok(())
}
pub fn pw_run() {}
