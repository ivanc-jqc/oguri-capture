use pipewire::core::CoreRc;
use pipewire::properties::properties;

use pipewire::spa::param::format_utils;
use pipewire::spa::param::format::FormatProperties;
use pipewire::spa::param::video::VideoInfoRaw;
use pipewire::spa::pod::{Value, deserialize};
use pipewire::spa::{
    param::{ParamType, video::VideoFormat},
    pod::{Object, Pod},
    utils::Direction,
};
use pipewire::stream::{StreamFlags, StreamListener, StreamRc};

use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Producer, Split};

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::{mem, num};
use std::path::Path;
use std::process::Command;

pub const FRAME_RATE: u64 = 60;
pub const WINDOW_SIZE: u64 = 1920*1080;
pub const CLIP_DURATION: u64 = 30;
pub const DEFAULT_CHANNELS: u32 = 2;
pub struct VideoData {
    format: VideoInfoRaw,
}
pub struct Video {
    video_core: CoreRc,
    video_streams: Vec<StreamRc>,
    audio_listeners: Vec<StreamListener<VideoData>>,
    pub audio_buffer_cons: ringbuf::HeapCons<u8>,
}
impl Video {
    pub fn new(video_core: CoreRc, node_id: u32) -> Self {
        let temp = HeapRb::<u8>::new((WINDOW_SIZE).try_into().unwrap());
        let video_stream = create_video_stream(video_core.clone()).unwrap();
        let (mut prod, mut cons) = temp.split();
        let listener = attach_audio_listener(video_stream.clone(), prod).unwrap();
        let video_stream_params_data = create_stream_params_bytes();
        let mut video_stream_params = [Pod::from_bytes(&video_stream_params_data).unwrap()];
        video_stream
            .connect(
                Direction::Input,
                Some(node_id), //which node to listen to
                StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
                &mut video_stream_params,
            )
            .unwrap();

        Self {
            video_core,
            video_streams: vec![video_stream],
            audio_listeners: vec![listener],
            audio_buffer_cons: cons,
        }
    }
    // pub fn dump(&mut self) {
    //     let path = Path::new("output/viddump");
    //     let mut file = File::create(path).unwrap();
    //     let bgrx_size: usize = 4;

    //     for _num_slices in 0..CLIP_DURATION*bgrx_size as u64 {

    //         //YO!!!! THIS CAN CAUSE STACK OVERFLOW!!!!
    //         let mut data = [0u8; (WINDOW_SIZE) as usize];
    //         self.audio_buffer_cons.pop_slice(&mut data);
    //         let num_of_pixels = WINDOW_SIZE as usize/bgrx_size;
    //         for x in 0..num_of_pixels {
    //             let start = x;
    //             let end = start + bgrx_size;
    //             let pixel_slice = &data[start..end];
    //             let new_slice = [pixel_slice[2], pixel_slice[1], pixel_slice[0]]; 
    //             println!("{:?}", new_slice);
    //         }
    //         //  file.write_all();
    //     }
    //     // let ffmpeg = Command::new("ffmpeg")
    //     //     .arg("-f")
    //     //     .arg("f32le")
    //     //     .arg("-ar")
    //     //     .arg("44100")
    //     //     .arg("-ac")
    //     //     .arg("2")
    //     //     .arg("-i")
    //     //     .arg("output/dump")
    //     //     .arg("output.wav")
    //     //     .arg("-y")
    //     //     .output()
    //     //     .expect("Failed to execute command");
    //     // println!("{:?}", ffmpeg.stdout);
    //     // for x in data.iter(){
    //     //     let temp = x.to_le_bytes();
    //     //

    //     // }
    // }
}

fn attach_audio_listener(
    stream: StreamRc,
    mut prod: ringbuf::HeapProd<u8>,
) -> Result<StreamListener<VideoData>, Box<dyn Error>> {
    let video_data = VideoData {
        format: Default::default(),
    };

    let listener = stream
        .add_local_listener_with_user_data(video_data)
        .state_changed(|_, _, old, new| {
            println!("State changed: {:?} -> {:?}", old, new);
        })
        .param_changed(|stream, user_data, id, param| {
            let Some(param) = param else {
                return;
            };
            dbg!(ParamType::from_raw(id));
            if id != pipewire::spa::param::ParamType::Format.as_raw() {
                return;
            }

            let (media_type, media_subtype) =
                match pipewire::spa::param::format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(_) => return,
                };

            if media_type != pipewire::spa::param::format::MediaType::Video
                || media_subtype != pipewire::spa::param::format::MediaSubtype::Raw
            {
                return;
            }
            // pipewire::spa::pod::deserialize::PodDeserializer::de(param);
            // let temp = param.as_object().unwrap();
            // for x in temp.props() {
            //     if x.key().0 == FormatProperties::VideoFramerate.0 {
            //         let val = x.value();
            //     }
            //     // dbg!(x.key());
            // }
            dbg!(user_data.format);
            
            user_data.format.parse(param).expect("OAJDSOAJDOA");
            // println!(
            //     "  size: {}x{}",
            //     user_data.format.size().width,
            //     user_data.format.size().height
            // );
            // println!(
            //     "  framerate: {}/{}",
            //     user_data.format.framerate().num,
            //     user_data.format.framerate().denom
            // );
        })
        .process(move |stream, video_data| match stream.dequeue_buffer() {
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
fn create_video_stream(video_core: CoreRc) -> Result<StreamRc, Box<dyn Error>> {
    let stream = StreamRc::new(
        video_core,
        "oguri-capture",
        properties! {
            "PW_KEY_MEDIA_TYPE" => "Video",
            "PW_KEY_MEDIA_CATEGORY" => "Capture",
            // "PW_KEY_MEDIA_ROLE" => "Playback"
        },
    )?;
    Ok(stream)
}

fn create_stream_params_bytes() -> Vec<u8> {
    let pod_object = pipewire::spa::pod::object!(
        pipewire::spa::utils::SpaTypes::ObjectParamFormat,
        pipewire::spa::param::ParamType::EnumFormat,
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::MediaType,
            Id,
            pipewire::spa::param::format::MediaType::Video
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pipewire::spa::param::format::MediaSubtype::Raw
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::VideoFormat,
            // Choice,
            // Enum,
            Id,
            // pipewire::spa::param::video::VideoFormat::RGB,
            // pipewire::spa::param::video::VideoFormat::RGB,
            // pipewire::spa::param::video::VideoFormat::RGBA,
            // pipewire::spa::param::video::VideoFormat::RGBx,
            pipewire::spa::param::video::VideoFormat::BGRx // pipewire::spa::param::video::VideoFormat::YUY2,
                                                           // pipewire::spa::param::video::VideoFormat::I420,
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            pipewire::spa::utils::Rectangle {
                width: 1920,
                height: 1080
            },
            pipewire::spa::utils::Rectangle {
                width: 1920,
                height: 1080
            },
            pipewire::spa::utils::Rectangle {
                width: 1920,
                height: 1080
            }
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pipewire::spa::utils::Fraction { num: 60, denom: 1 },
            pipewire::spa::utils::Fraction { num: 0, denom: 1 },
            pipewire::spa::utils::Fraction {
                num: 60,
                denom: 1
            }
        ),
    );

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
