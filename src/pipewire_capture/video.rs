use crate::circular_buffer::CircularBuffer;
use pipewire::core::CoreRc;
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
use pipewire::stream::{StreamListener, StreamRc, StreamFlags};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 1;
pub struct VideoData {
    format: VideoInfoRaw,
    write_buffer: Rc<RefCell<CircularBuffer<f32>>>,
}
pub struct Video {
    pub video_core: CoreRc,
    pub video_streams: RefCell<Vec<StreamRc>>,
    pub video_listeners: RefCell<Vec<StreamListener<VideoData>>>,
    pub video_buffer: Rc<RefCell<CircularBuffer<f32>>>,
}
impl Video {
    pub fn new(video_core: CoreRc) -> Self {
        Self {
            video_core:video_core,
            video_streams: RefCell::new(Vec::new()),
            video_listeners: RefCell::new(Vec::new()),
            video_buffer: Rc::new(RefCell::new(CircularBuffer::new(
                (DEFAULT_RATE * 30).try_into().unwrap()),
            )),
        }
    }
    pub fn connect_audio(&self) -> Result<(), Box<dyn Error>> {
        let audio_stream = create_audio_stream(self.video_core.clone()).unwrap();
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
        self.video_streams.borrow_mut().push(audio_stream);
        self.video_listeners.borrow_mut().push(listener);
        Ok(())
    }
    fn attach_audio_listener(
        &self,
        stream: StreamRc,
    ) -> Result<StreamListener<VideoData>, Box<dyn Error>> {
        let audio_data = VideoData {
            format: Default::default(),
            write_buffer: self.video_buffer.clone(),
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
}


//probably turn these into traits
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
