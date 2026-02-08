use pipewire::core::CoreRc;
use pipewire::properties::properties;

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
use ringbuf::traits::{Consumer, Producer, Split};

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::mem;
use std::process::Command;

pub const DEFAULT_RATE: u32 = 44100;
pub const DEFAULT_CHANNELS: u32 = 2;
pub struct AudioData {
    format: AudioInfoRaw,
}
pub struct Audio {
    audio_core: CoreRc,
    audio_streams: Vec<StreamRc>,
    audio_listeners: Vec<StreamListener<AudioData>>,
    pub audio_buffer_cons: ringbuf::HeapCons<u8>,
}
impl Audio {
    pub fn new(audio_core: CoreRc) -> Self {
        let temp = HeapRb::<u8>::new((DEFAULT_RATE * 30).try_into().unwrap());
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
        let f32_size = mem::size_of::<f32>();

        let mut data = [0u8; (DEFAULT_RATE * 30) as usize];
        self.audio_buffer_cons.pop_slice(&mut data);

        let path = Path::new("output/dump");
        let mut file = File::create(path).unwrap();
        
        let n_samples = data.len()/f32_size;
        
        for i in 0..n_samples {
            let start = i as usize * f32_size;
            let end = start + f32_size;
            let slice = &data[start..end];
            let sample = f32::from_le_bytes(slice.try_into().unwrap());
            // println!("{}", sample);
            let temp = sample.to_le_bytes();
            file.write_all(&temp).unwrap();
        }
        let ffmpeg = Command::new("ffmpeg")
            .arg("-f")
            .arg("f32le")
            .arg("-ar")
            .arg("44100")
            .arg("-ac")
            .arg("2")
            .arg("-i")
            .arg("output/dump")
            .arg("output.wav")
            .arg("-y")
            .output()
            .expect("Failed to execute command");
        println!("{:?}", ffmpeg.stdout);
        // for x in data.iter(){
        //     let temp = x.to_le_bytes();
        //     
    
        // }
    }
}

fn attach_audio_listener(stream: StreamRc, mut prod: ringbuf::HeapProd<u8>) -> Result<StreamListener<AudioData>, Box<dyn Error>> {
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
                let size = data.chunk().size() as usize;
                let bytes = data.data().unwrap();
                prod.push_slice(&bytes[0..size]);
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
    let mut position = [0; pipewire::spa::param::audio::MAX_CHANNELS];
    position[0] = libspa_sys::SPA_AUDIO_CHANNEL_FL;
    position[1] = libspa_sys::SPA_AUDIO_CHANNEL_FR;
    audio_info.set_position(position);

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
