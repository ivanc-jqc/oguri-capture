use crate::circular_buffer::CircularBuffer;
use futures_util::lock::Mutex;
use pipewire::core::CoreRc;
use pipewire::properties::properties;
use pipewire::spa::param::audio;
use pipewire::spa::{
    param::{
        ParamType,
        audio::{AudioFormat, AudioInfoRaw},
    },
    pod::{Object, Pod},
    utils::Direction,
};
use pipewire::stream::{StreamFlags, StreamListener, StreamRc};
use ringbuf::HeapRb;
use ringbuf::storage::Heap;
use ringbuf::traits::{Consumer, Producer, Split};
use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::thread;
use std::time::Duration;
pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 1;
pub struct AudioData {
    format: AudioInfoRaw,
}
pub struct Audio {
    audio_core: CoreRc,
    audio_streams: Vec<StreamRc>,
    audio_listeners: Vec<StreamListener<AudioData>>,
    pub audio_buffer_cons: ringbuf::HeapCons<f32>,
}
impl Audio {
    pub fn new(audio_core: CoreRc) -> Self {
        let temp = HeapRb::<f32>::new((DEFAULT_RATE * 30).try_into().unwrap());
        let audio_stream = create_audio_stream(audio_core.clone()).unwrap();
        let (mut prod, mut cons) = temp.split();
        let listener = attach_audio_listener(audio_stream.clone(),prod).unwrap();
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

        Self {
            audio_core: audio_core,
            audio_streams: vec![audio_stream],
            audio_listeners: vec![listener],
            audio_buffer_cons: cons,
        }
    }
    pub fn dump(&mut self) {
        let mut data = vec![0f32; (DEFAULT_RATE * 30).try_into().unwrap()];
        self.audio_buffer_cons.pop_slice(&mut data);
        let path = Path::new("output/dump");
        let mut file = File::create(path).unwrap();
        for x in data.iter(){
            let temp = x.to_le_bytes();
            file.write_all(&temp).unwrap();
    
        }
    }
}

fn attach_audio_listener(stream: StreamRc, mut prod: ringbuf::HeapProd<f32>) -> Result<StreamListener<AudioData>, Box<dyn Error>> {
    let audio_data = AudioData {
        format: Default::default(),
    };

    let listener = stream
        .add_local_listener_with_user_data(audio_data)
        .param_changed(|stream, user_data, id, param| {
            let Some(param) = param else {
                return;
            };
        })
        .process(move |stream, audio_data| match stream.dequeue_buffer() {
            None => println!("No buffer received"),
            Some(mut buffer) => {
                let datas = buffer.datas_mut();
                let mut data = &mut datas[0];
                let bytes = data.data().unwrap();
                let slice = &bytes[0..4];
                let sample = f32::from_le_bytes(slice.try_into().unwrap());
                prod.try_push(sample);
                // {
                //     let mut x = audio_data.write_buffer.push(sample);
                //     drop(x);
                // }
                // println!("{}", sample);
            }
        })
        .register()?;
    Ok(listener)
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
